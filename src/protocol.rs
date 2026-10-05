use crate::constants::{H, I, X};
use crate::registry::Qubit;
use crate::rng::{rand_float, shuffle_and_split};
use crate::participants::{Receiver, Sender};
use bon::Builder;
use statrs::distribution::{ContinuousCDF, Normal};

#[cfg(target_arch = "wasm32")]
use web_time::Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

/// Represents the result of a single quantum execution round in a QKD protocol.
///
/// This struct captures the values and bases chosen by Alice and Bob,
/// as well as the potential eavesdropping attempts by Eve during the round.
#[derive(Clone, Debug)]
pub struct QExecutionResult {
    /// Bit value chosen by Alice.
    pub alice_value: bool,
    /// Basis used by Alice for her prepared qubit.
    pub alice_basis: usize,
    /// Bit value measured by Bob.
    pub bob_value: bool,
    /// Basis used by Bob for his measurement.
    pub bob_basis: usize,
    /// Bit value potentially intercepted by Eve, if any.
    pub eve_value: Option<bool>,
    /// Measurement basis used by Eve, if any.
    pub eve_basis: Option<usize>,
}

impl QExecutionResult {
    /// Creates a new `QExecutionResult` with the specified values and bases.
    ///
    /// # Arguments
    ///
    /// * `alice_value` - Alice's bit value.
    /// * `alice_basis` - Basis used by Alice.
    /// * `bob_value` - Bob's measured bit value.
    /// * `bob_basis` - Basis used by Bob.
    /// * `eve_value` - Eve's intercepted bit value, if any.
    /// * `eve_basis` - Basis used by Eve, if any.
    pub fn new(
        alice_value: bool,
        alice_basis: usize,
        bob_value: bool,
        bob_basis: usize,
        eve_value: Option<bool>,
        eve_basis: Option<usize>,
    ) -> Self {
        QExecutionResult {
            alice_value,
            alice_basis,
            bob_value,
            bob_basis,
            eve_value,
            eve_basis,
        }
    }
}

/// Represents the result of a Quantum Key Distribution (QKD) protocol execution.
///
/// This struct encapsulates the outcome of the entire QKD process, including
/// timing, security status, key metrics, and estimated eavesdropping knowledge.
#[derive(Debug)]
pub struct QKDResult {
    /// Total duration of the quantum communication process, from initialization to completion.
    pub elapsed_time: u128,

    /// Indicates whether the communication is considered secure.
    /// If `false`, the protocol was aborted due to security concerns.
    pub is_considered_secure: bool,

    /// Length of the final generated key in bits.
    /// If the protocol is aborted, this is `None`.
    pub key_length: Option<usize>,

    /// Quantum Bit Error Rate (QBER) of the final generated key.
    /// If the protocol is aborted, this is `None`.
    pub final_key_qber: Option<f64>,

    /// Quantum Bit Error Rate (QBER) of the publicly disclosed values, i.e. the fraction of
    /// disclosed bits where Alice and Bob disagree. Compared against a threshold derived from
    /// `noise` and `confidence` to decide `is_considered_secure` — see [`QKD::run`] for how
    /// that threshold is computed and why it becomes exact-equality-to-`noise` when
    /// `noise == 0.0`.
    pub measured_qber: f64,

    /// Estimated fraction of the final key known by an eavesdropper (Eve).
    ///
    /// This only counts a key bit as "known by Eve" when Eve intercepted that round *and*
    /// Alice's and Bob's final values for it agree (i.e. it did not contribute to
    /// `final_key_qber`) *and* Eve's measured value matches theirs. Bits where Alice and Bob
    /// disagree are excluded from the count of bits Eve is credited with knowing, even though
    /// Eve may have measured them correctly — that disagreement could stem from noise or from
    /// Eve's own interference, and the two cases are not distinguished. Consequently, this
    /// value is a **lower bound** on Eve's true knowledge whenever `final_key_qber > 0.0`, not
    /// an exact count.
    ///
    /// Also note this field is only meaningful when `is_considered_secure` is `true`: when the
    /// protocol is aborted, `eve_knowledge` is left at its default `0.0` rather than becoming
    /// `None` (unlike `key_length` and `final_key_qber`), so a bare `0.0` does not necessarily
    /// mean Eve learned nothing — check `is_considered_secure` first.
    pub eve_knowledge: f64,
}

/// Represents the public discussion phase results of a QKD protocol.
///
/// This struct contains the values publicly shared by Alice and Bob,
/// the indexes of the bits used to generate the final key,
/// and the detailed results of each quantum execution round.
#[derive(Debug)]
pub struct PublicDiscussionResult {
    /// Publicly announced bit values by Alice.
    pub alice_public_values: Vec<bool>,
    /// Publicly announced bit values by Bob.
    pub bob_public_values: Vec<bool>,
    /// Indexes of the bits selected for the final key generation.
    pub indexes_to_key: Vec<usize>,
    /// Detailed results of each quantum execution round.
    pub results: Vec<QExecutionResult>,
}

/// Represents a Quantum Key Distribution (QKD) protocol instance.
///
/// This struct encapsulates the participants (Alice, Bob, and Eve),
/// the public basis discussion logic, and the methods to execute the protocol.
#[derive(Builder)]
pub struct QKD {
    /// The name of the QKD protocol.
    name: String,
    /// Quantum sender (Alice) in the QKD protocol.
    alice: Sender,
    /// Quantum receiver (Bob) in the QKD protocol.
    bob: Receiver,
    /// Potential eavesdropper (Eve) in the QKD protocol.
    /// By default, Eve can measure in the I and H bases.
    #[builder(default = Receiver::builder().posible_basis(vec![I, H]).build())]
    eve: Receiver,
    /// Function to perform the public basis discussion phase.
    /// Determines which bits are used for key generation and which for security checking.
    #[builder(default = Box::new(default_public_basis_discussion))]
    public_basis_discussion: Box<dyn Fn(&Vec<QExecutionResult>) -> PublicDiscussionResult>,
    /// Function that decides, from the public discussion's disclosed values (and
    /// `results`, for protocols that need more than just the disclosed bits — e.g. a
    /// CHSH-based check that needs to know which basis pair produced each value),
    /// whether the run is secure, and the measured error/statistic to report.
    ///
    /// By default (`default_check_security`), this is the QBER-vs-`noise` threshold
    /// test documented on [`QKD::run`]. A protocol whose security proof is not based on
    /// a simple error threshold (e.g. a Bell/CHSH inequality test) can override this
    /// instead of trying to repurpose the QBER test for something it wasn't designed
    /// to measure.
    #[builder(default = Box::new(default_check_security))]
    check_security: Box<dyn Fn(&PublicDiscussionResult, f64, f64) -> (bool, f64)>,
}

impl QKD {
    /// Returns the name of the QKD protocol instance.
    ///
    /// # Returns
    /// A `String` representing the name of the protocol.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Executes the QKD protocol for a given number of qubits and interception rate.
    ///
    /// # Arguments
    ///
    /// * `number_of_qubits` - Number of qubits to use in the protocol.
    /// * `interception_rate` - Probability (0.0 to 1.0) that Eve intercepts a qubit.
    /// * `noise` - Expected bit-flip probability of the channel in the absence of
    ///   eavesdropping (the null hypothesis tested against the publicly disclosed values).
    ///   This is not just a lower bound on the tolerated error rate: it is the exact value
    ///   the statistical test assumes to be true, and its own uncertainty scales with
    ///   `sqrt(noise * (1.0 - noise))`.
    ///   **Important:** when `noise == 0.0`, that uncertainty term is exactly `0.0`, so the
    ///   accepted error threshold collapses to `0.0` regardless of `confidence` — any single
    ///   mismatch between Alice's and Bob's publicly disclosed values will make the protocol
    ///   be considered insecure. This is intentional (a channel declared to have zero noise
    ///   is expected to never produce an error), but it means `confidence` has no smoothing
    ///   effect at `noise = 0.0`. Use a small positive `noise` (e.g. `0.01`) to model a
    ///   realistic imperfect channel and let `confidence` provide real tolerance.
    /// * `confidence` - Statistical confidence level (0.0 to 1.0, e.g. `0.99`) for the
    ///   one-sided test of whether the measured QBER is consistent with `noise`. Only has an
    ///   effect when `noise > 0.0` (see above).
    ///
    /// # Returns
    ///
    /// A `QKDResult` containing the protocol outcome, including timing,
    /// security status, key metrics, and estimated eavesdropping knowledge.
    pub fn run(
        &self,
        number_of_qubits: usize,
        interception_rate: f64,
        noise: f64,
        confidence: f64,
    ) -> QKDResult {
        assert!(
            !self.alice.posible_basis.is_empty(),
            "Alice's posible_basis must not be empty"
        );
        assert!(
            !self.bob.posible_basis.is_empty(),
            "Bob's posible_basis must not be empty"
        );
        assert!(
            !self.eve.posible_basis.is_empty(),
            "Eve's posible_basis must not be empty"
        );

        let initial_time = Instant::now();
        let results = (0..number_of_qubits)
            .map(|_| self.quantum_communication(interception_rate, noise))
            .collect::<Vec<QExecutionResult>>();

        let discussion_result = (self.public_basis_discussion)(&results);
        let (is_considered_secure, measured_qber) =
            (self.check_security)(&discussion_result, noise, confidence);
        let results = discussion_result.results;

        let mut eve_knowledge = 0.0;
        let mut final_key_qber = None;
        let mut key_length = None;
        if is_considered_secure {
            let ((alice_secret_values, bob_secret_values), eve_secret_values): (
                (Vec<bool>, Vec<bool>),
                Vec<Option<bool>>,
            ) = discussion_result
                .indexes_to_key
                .iter()
                .map(|&i| {
                    (
                        (results[i].alice_value, results[i].bob_value),
                        results[i].eve_value,
                    )
                })
                .unzip();

            key_length = Some(alice_secret_values.len());

            // For each final-key bit: if Alice and Bob disagree, count it towards
            // `final_key_qber` only — Eve's value is not consulted, so an eavesdropped bit
            // that also happens to be a mismatch is *not* credited to `eve_knowledge` (see
            // the doc comment on `QKDResult::eve_knowledge` for why this makes it a lower
            // bound rather than an exact count).
            let (mismatched_bits, absolute_eve_knowledge) = alice_secret_values
                .into_iter()
                .zip(bob_secret_values)
                .zip(eve_secret_values)
                .fold((0.0, 0.0), |mut acc, ((a, b), e)| {
                    if a != b {
                        acc.0 += 1.0;
                    } else {
                        acc.1 += if e == Some(a) { 1.0 } else { 0.0 }
                    }
                    acc
                });

            final_key_qber = Some(mismatched_bits / key_length.unwrap() as f64);
            eve_knowledge = absolute_eve_knowledge / key_length.unwrap() as f64;
        }
        let elapsed_time = initial_time.elapsed().as_micros();

        QKDResult {
            elapsed_time,
            is_considered_secure,
            key_length,
            measured_qber,
            final_key_qber,
            eve_knowledge,
        }
    }

    /// Simulates a single quantum communication round between Alice and Bob,
    /// with potential eavesdropping by Eve.
    ///
    /// # Arguments
    ///
    /// * `interception_rate` - Probability (0.0 to 1.0) that Eve intercepts the qubit.
    ///
    /// # Returns
    ///
    /// A `QExecutionResult` containing the values and bases chosen by Alice, Bob, and Eve.
    fn quantum_communication(&self, interception_rate: f64, noise: f64) -> QExecutionResult {
        // Clears every QuantumState created on the previous round, so the registry
        // never grows past what a single round needs regardless of how many rounds
        // `run` executes overall.
        Qubit::reset_registry();

        // Alice
        let (qubit, alice_value, alice_basis) = (self.alice.prepare)(&self.alice.posible_basis);

        // Eve
        let mut eve_basis = None;
        let mut eve_value = None;
        if rand_float() < interception_rate {
            eve_basis = Some((self.eve.change_basis)(&qubit, &self.eve.posible_basis));
            eve_value = Some((self.eve.measure)(&qubit));
            (self.eve.try_to_restore_qubit)(&qubit, &self.eve.posible_basis[eve_basis.unwrap()]);
        }

        // Perform bit-flip
        if rand_float() < noise {
            qubit.apply_local_gate(&X);
        }

        // Bob
        let bob_basis = (self.bob.change_basis)(&qubit, &self.bob.posible_basis);
        let bob_value = (self.bob.measure)(&qubit);

        QExecutionResult::new(
            alice_value,
            alice_basis,
            bob_value,
            bob_basis,
            eve_value,
            eve_basis,
        )
    }

}

/// Default security check: verifies the publicly disclosed values are consistent with
/// the expected channel `noise`, at the given statistical `confidence` level.
///
/// This performs a one-sided hypothesis test: under the null hypothesis that the true
/// per-bit error probability equals `noise`, the accepted upper bound on the observed
/// QBER is `noise + z * sqrt(noise * (1 - noise) / sample_size)`, where `z` is the
/// `confidence`-quantile of the standard normal distribution. The measured QBER is
/// compared against that bound.
///
/// Note the term `sqrt(noise * (1 - noise))` is exactly `0.0` when `noise` is `0.0` or
/// `1.0`: at those boundary values the hypothesis has no variance, so the threshold
/// collapses to `noise` itself and `confidence` has no effect (see [`QKD::run`]).
///
/// # Arguments
///
/// * `discussion` - The public discussion result; only the disclosed values are used.
/// * `noise` - Expected per-bit error probability under the "no eavesdropping" hypothesis.
/// * `confidence` - Statistical confidence level for the test.
///
/// # Returns
///
/// A tuple `(is_considered_secure, measured_qber)`. If there are no publicly disclosed
/// values to compare (`sample_size == 0`), the protocol cannot be statistically validated
/// and this returns `(false, 0.0)`.
fn default_check_security(
    discussion: &PublicDiscussionResult,
    noise: f64,
    confidence: f64,
) -> (bool, f64) {
    let sample_size = discussion.alice_public_values.len() as f64;
    // No publicly disclosed values means there is nothing to statistically
    // validate, so the protocol cannot be considered secure. Without this
    // guard, `sample_size == 0.0` would produce NaN/inf below.
    if sample_size == 0.0 {
        return (false, 0.0);
    }

    let normal = Normal::standard();

    let p = (1.0 + confidence) / 2.0;
    let z = normal.inverse_cdf(p);

    let threshold = noise + z / sample_size.sqrt() * (noise * (1.0 - noise)).sqrt();

    let measured_qber = discussion
        .alice_public_values
        .iter()
        .zip(discussion.bob_public_values.iter())
        .filter(|(a, b)| a != b)
        .count() as f64
        / sample_size;

    (measured_qber <= threshold, measured_qber)
}

/// Default public basis discussion function.
///
/// Selects a random subset of matching basis results for public comparison,
/// and uses the remaining for key generation.
///
/// # Arguments
///
/// * `results` - Vector of quantum execution results.
///
/// # Returns
///
/// A `PublicDiscussionResult` containing the public values, key indexes, and results.
fn default_public_basis_discussion(results: &Vec<QExecutionResult>) -> PublicDiscussionResult {
    let (alice_basis, bob_basis): (Vec<usize>, Vec<usize>) =
        results.iter().map(|x| (x.alice_basis, x.bob_basis)).unzip();

    let eq_basis_indexes = alice_basis
        .into_iter()
        .zip(bob_basis)
        .enumerate()
        .filter(|(_, (a, b))| a == b)
        .map(|(i, _)| i)
        .collect::<Vec<usize>>();

    let (indexes_to_check, indexes_to_key) = shuffle_and_split(eq_basis_indexes);

    let (alice_public_values, bob_public_values) = indexes_to_check
        .iter()
        .map(|&i| (results[i].alice_value, results[i].bob_value))
        .unzip();

    PublicDiscussionResult {
        alice_public_values,
        bob_public_values,
        indexes_to_key,
        results: results.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::participants::{Receiver, Sender};
    use crate::rng::set_global_seed;

    fn minimal_qkd() -> QKD {
        let alice = Sender::builder().posible_basis(vec![I]).build();
        let bob = Receiver::builder().posible_basis(vec![I]).build();
        QKD::builder()
            .name("Test".to_string())
            .alice(alice)
            .bob(bob)
            .build()
    }

    fn discussion(alice_public_values: Vec<bool>, bob_public_values: Vec<bool>) -> PublicDiscussionResult {
        PublicDiscussionResult {
            alice_public_values,
            bob_public_values,
            indexes_to_key: vec![],
            results: vec![],
        }
    }

    #[test]
    fn check_security_empty_sample_is_insecure_with_zero_qber() {
        let (secure, qber) = default_check_security(&discussion(vec![], vec![]), 0.0, 0.99);
        assert!(!secure);
        assert_eq!(qber, 0.0);
        assert!(!qber.is_nan());
    }

    #[test]
    fn check_security_no_mismatches_is_secure() {
        let values = vec![true, false, true, false, true];
        let (secure, qber) = default_check_security(&discussion(values.clone(), values), 0.0, 0.99);
        assert!(secure);
        assert_eq!(qber, 0.0);
    }

    #[test]
    fn check_security_any_mismatch_is_insecure_when_noise_is_zero() {
        let alice_values = vec![true; 1000];
        let mut bob_values = alice_values.clone();
        bob_values[0] = false; // a single mismatch out of 1000
        let (secure, qber) =
            default_check_security(&discussion(alice_values, bob_values), 0.0, 0.5);
        assert!(!secure);
        assert!((qber - 0.001).abs() < 1e-9);
    }

    #[test]
    fn check_security_confidence_has_no_effect_when_noise_is_zero() {
        let alice_values = vec![true; 100];
        let bob_values = alice_values.clone();
        let (secure_low, _) = default_check_security(
            &discussion(alice_values.clone(), bob_values.clone()),
            0.0,
            0.5,
        );
        let (secure_high, _) =
            default_check_security(&discussion(alice_values, bob_values), 0.0, 0.9999999999);
        assert_eq!(secure_low, secure_high);
    }

    #[test]
    fn check_security_zero_confidence_gives_zero_tolerance_above_noise() {
        // confidence = 0.0 -> z = inverse_cdf(0.5) = 0.0, so the threshold collapses to
        // exactly `noise`, with no statistical margin at all -- mirroring the
        // noise = 0.0 edge case, but for a different reason (z = 0 instead of zero
        // variance). A 6% observed error rate against a 5% declared noise is a
        // plausible sampling fluctuation that a normal confidence level accepts, but
        // confidence = 0.0 rejects outright.
        let n = 1000;
        let mismatches = 60; // 6% observed vs 5% declared noise
        let alice_values = vec![true; n];
        let mut bob_values = alice_values.clone();
        for value in bob_values.iter_mut().take(mismatches) {
            *value = false;
        }
        let (secure_zero_confidence, _) = default_check_security(
            &discussion(alice_values.clone(), bob_values.clone()),
            0.05,
            0.0,
        );
        let (secure_normal_confidence, _) =
            default_check_security(&discussion(alice_values, bob_values), 0.05, 0.99);
        assert!(!secure_zero_confidence);
        assert!(secure_normal_confidence);
    }

    #[test]
    fn check_security_tolerates_expected_noise_within_confidence() {
        let n = 2000;
        let mismatches = 100; // 5% observed, matching the 5% declared noise
        let alice_values = vec![true; n];
        let mut bob_values = alice_values.clone();
        for value in bob_values.iter_mut().take(mismatches) {
            *value = false;
        }
        let (secure, qber) =
            default_check_security(&discussion(alice_values, bob_values), 0.05, 0.99);
        assert!(secure);
        assert!((qber - 0.05).abs() < 1e-9);
    }

    #[test]
    fn check_security_threshold_matches_the_documented_formula() {
        // Independently recompute `noise + z * sqrt(noise * (1 - noise) / n)` (the
        // formula documented on this function) and probe a QBER just below and just
        // above it, to catch any accidental change to the arithmetic (e.g. a stray
        // `*` <-> `/` or `-` <-> `+` swap) that a loose "still secure" assertion would
        // not notice.
        let noise: f64 = 0.4;
        let confidence: f64 = 0.95;
        let n: usize = 1000;

        let normal = Normal::standard();
        let z = normal.inverse_cdf((1.0 + confidence) / 2.0);
        let expected_threshold = noise + z / (n as f64).sqrt() * (noise * (1.0 - noise)).sqrt();

        let make_values = |qber: f64| {
            let mismatches = (qber * n as f64).round() as usize;
            let alice_values = vec![true; n];
            let mut bob_values = alice_values.clone();
            for value in bob_values.iter_mut().take(mismatches) {
                *value = false;
            }
            (alice_values, bob_values)
        };

        let (alice_below, bob_below) = make_values(expected_threshold - 0.005);
        let (secure_below, _) =
            default_check_security(&discussion(alice_below, bob_below), noise, confidence);
        assert!(secure_below, "expected secure just below the computed threshold");

        let (alice_above, bob_above) = make_values(expected_threshold + 0.005);
        let (secure_above, _) =
            default_check_security(&discussion(alice_above, bob_above), noise, confidence);
        assert!(
            !secure_above,
            "expected insecure just above the computed threshold"
        );
    }

    #[test]
    #[should_panic(expected = "Alice's posible_basis must not be empty")]
    fn run_panics_clearly_when_alice_basis_is_empty() {
        let alice = Sender::builder().posible_basis(vec![]).build();
        let bob = Receiver::builder().posible_basis(vec![I]).build();
        let qkd = QKD::builder()
            .name("Bad".to_string())
            .alice(alice)
            .bob(bob)
            .build();
        qkd.run(10, 0.0, 0.0, 0.99);
    }

    #[test]
    #[should_panic(expected = "Bob's posible_basis must not be empty")]
    fn run_panics_clearly_when_bob_basis_is_empty() {
        let alice = Sender::builder().posible_basis(vec![I]).build();
        let bob = Receiver::builder().posible_basis(vec![]).build();
        let qkd = QKD::builder()
            .name("Bad".to_string())
            .alice(alice)
            .bob(bob)
            .build();
        qkd.run(10, 0.0, 0.0, 0.99);
    }

    #[test]
    fn run_with_zero_qubits_does_not_produce_nan() {
        set_global_seed(1);
        let qkd = minimal_qkd();
        let result = qkd.run(0, 0.0, 0.0, 0.9999999999);
        assert!(!result.measured_qber.is_nan());
        assert!(!result.is_considered_secure);
        assert_eq!(result.key_length, None);
        assert_eq!(result.final_key_qber, None);
    }
}
