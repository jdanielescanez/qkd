/// Module containing the implementation of QKD protocol participants (Alice, Bob, and Eve).
/// Provides structs and builders for creating and configuring participants with their
/// respective quantum bases and behaviors.
pub mod participants;

/// Module implementing the core Quantum Key Distribution protocols.
/// Contains the main QKD struct, protocol execution logic, and result types
/// including QKDResult and PublicDiscussionResult.
pub mod protocol;

/// Module defining fundamental quantum types and structures.
/// Includes `ComplexMatrix` and `QuantumState`, the joint-state representation that
/// backs every qubit (entangled or not) used throughout the QKD simulations.
mod types;

/// Module providing the registry-backed `Qubit` handle used to represent qubits that
/// may be part of a larger (possibly entangled) `QuantumState`.
mod registry;

/// Module containing fundamental quantum constant matrices.
/// Provides predefined quantum gates and operations used in QKD protocols,
/// including identity (I), Hadamard (H), Pauli-X (X), and Y-basis Hadamard (H_Y) matrices.
pub mod constants;

/// Module providing the global seedable random number generator.
/// Call `set_global_seed` for reproducible simulations.
pub mod rng;

use constants::{rotation, H, H_Y, I};
pub use participants::{Receiver, Sender};
pub use protocol::{PublicDiscussionResult, QExecutionResult, QKDResult, QKD};
pub use registry::Qubit;
pub use rng::set_global_seed;
use rng::{rand_choose, shuffle_and_split};
use statrs::distribution::{ContinuousCDF, Normal};
use std::f64::consts::PI;
pub use types::ComplexMatrix;

/// Builds and configures a QKD instance for the BB84 protocol.
///
/// # Returns
/// A `QKD` instance configured with Alice and Bob using the I and H bases.
pub fn build_bb84() -> QKD {
    let alice = Sender::builder().posible_basis(vec![I, H]).build();
    let bob = Receiver::builder().posible_basis(vec![I, H]).build();

    QKD::builder()
        .name("BB84".to_string())
        .alice(alice)
        .bob(bob)
        .build()
}

/// Builds and configures a QKD instance for the Six-State protocol.
///
/// # Returns
/// A `QKD` instance configured with Alice, Bob, and Eve using the I, H, and H_Y bases.
pub fn build_six_state() -> QKD {
    let alice = Sender::builder().posible_basis(vec![I, H, H_Y]).build();
    let bob = Receiver::builder()
        .posible_basis(vec![I, H, H_Y.invert().unwrap()])
        .build();
    let eve = Receiver::builder()
        .posible_basis(vec![I, H, H_Y.invert().unwrap()])
        .build();

    QKD::builder()
        .name("SixState".to_string())
        .alice(alice)
        .bob(bob)
        .eve(eve)
        .build()
}

/// Builds and configures a QKD instance for the B92 protocol.
///
/// # Returns
/// A `QKD` instance configured with Alice and Bob using the I and H bases,
/// and a custom public basis discussion function for the B92 protocol.
pub fn build_b92() -> QKD {
    let prepare_b92 = Box::new(|posible_basis: &Vec<ComplexMatrix>| {
        let qubit = Qubit::create_basis_state(false);
        let (basis_id, matrix) = rand_choose(posible_basis.iter().enumerate().collect());
        qubit.apply_local_gate(matrix);
        (qubit, false, basis_id)
    });

    let alice = Sender::builder()
        .posible_basis(vec![I, H])
        .prepare(prepare_b92)
        .build();
    let bob = Receiver::builder().posible_basis(vec![I, H]).build();

    QKD::builder()
        .name("B92".to_string())
        .alice(alice)
        .bob(bob)
        .public_basis_discussion(Box::new(public_basis_discussion_b92))
        .build()
}

/// Builds and configures a QKD instance for the BBM92 protocol: an entanglement-based
/// variant of BB84. Instead of Alice preparing and sending a qubit, a Bell pair is
/// created each round; Alice measures her own half (in a randomly chosen basis) and
/// Bob's half — already correctly collapsed by entanglement to match Alice's basis
/// and value — continues through the same pipeline as every other protocol (Eve may
/// intercept it, channel noise may flip it, Bob measures it). The public discussion
/// and security check are identical to BB84's, since the classical post-processing
/// only depends on bases and values, not on how the qubit was produced.
///
/// # Returns
/// A `QKD` instance configured with Alice and Bob using the I and H bases.
pub fn build_bbm92() -> QKD {
    let alice = Sender::builder()
        .posible_basis(vec![I, H])
        .prepare(Box::new(prepare_bbm92))
        .build();
    let bob = Receiver::builder().posible_basis(vec![I, H]).build();

    QKD::builder()
        .name("BBM92".to_string())
        .alice(alice)
        .bob(bob)
        .build()
}

fn prepare_bbm92(posible_basis: &Vec<ComplexMatrix>) -> (Qubit, bool, usize) {
    let (alice_qubit, bob_qubit) = Qubit::create_entangled_state();
    let (basis_id, matrix) = rand_choose(posible_basis.iter().enumerate().collect());
    alice_qubit.apply_local_gate(matrix);
    let alice_value = alice_qubit.measure();
    (bob_qubit, alice_value, basis_id)
}

/// Alice's measurement angles for E91 (indices 0, 1, 2): 0°, 45°, 22.5°.
const E91_ALICE_ANGLES: [f64; 3] = [0.0, PI / 4.0, PI / 8.0];
/// Bob's measurement angles for E91 (indices 0, 1, 2): 0°, 22.5°, 67.5°.
const E91_BOB_ANGLES: [f64; 3] = [0.0, PI / 8.0, 3.0 * PI / 8.0];

/// `(alice_basis, bob_basis)` index pairs where Alice's and Bob's angles coincide
/// exactly (0° == 0°, and 22.5° == 22.5°), giving perfect correlation — these are the
/// pairs used for key generation.
const E91_KEY_PAIRS: [(usize, usize); 2] = [(0, 0), (2, 1)];

/// `(alice_basis, bob_basis)` index pairs used for the CHSH test, chosen so that the
/// angle differences are exactly the combination that maximizes the CHSH violation
/// for the Bell state this crate produces (see `check_security_e91`'s doc comment for
/// the derivation): `S = E(0,1) - E(0,2) + E(1,1) + E(1,2)`.
const E91_CHSH_PAIRS: [(usize, usize); 4] = [(0, 1), (0, 2), (1, 1), (1, 2)];

/// Builds and configures a QKD instance for the E91 protocol (Ekert 1991): an
/// entanglement-based protocol whose security proof is the violation of the CHSH
/// (Bell) inequality, rather than a simple error-rate threshold.
///
/// Alice and Bob each randomly choose one of 3 measurement angles per round
/// (`E91_ALICE_ANGLES`/`E91_BOB_ANGLES`, applied via [`constants::rotation`]). Two
/// angle-index combinations coincide exactly and are used for key generation
/// (`E91_KEY_PAIRS`); four more are used for the CHSH test (`E91_CHSH_PAIRS`); the
/// remaining combinations are discarded. See `check_security_e91` for how the CHSH
/// parameter `S` is computed and turned into a security decision.
///
/// Note `QKDResult::measured_qber` holds `|S|` for this protocol, not a bit error
/// rate — the field is shared across all protocols built on top of `QKD`, and E91's
/// natural statistic is `S`, not a QBER.
///
/// # Returns
/// A `QKD` instance configured with Alice and Bob using the E91 measurement angles.
pub fn build_e91() -> QKD {
    let alice = Sender::builder()
        .posible_basis(E91_ALICE_ANGLES.iter().map(|&theta| rotation(theta)).collect())
        .prepare(Box::new(prepare_bbm92))
        .build();
    let bob = Receiver::builder()
        .posible_basis(E91_BOB_ANGLES.iter().map(|&theta| rotation(theta)).collect())
        .build();
    let eve = Receiver::builder()
        .posible_basis(E91_BOB_ANGLES.iter().map(|&theta| rotation(theta)).collect())
        .build();

    QKD::builder()
        .name("E91".to_string())
        .alice(alice)
        .bob(bob)
        .eve(eve)
        .public_basis_discussion(Box::new(public_basis_discussion_e91))
        .check_security(Box::new(check_security_e91))
        .build()
}

/// E91's public discussion: classifies every round by its `(alice_basis, bob_basis)`
/// index pair into key-generating (`E91_KEY_PAIRS`) or discarded. Unlike the default
/// discussion, the publicly disclosed values (`alice_public_values`/`bob_public_values`)
/// are left empty — `check_security_e91` recomputes everything it needs directly from
/// `results`, since the CHSH test needs each round's basis pair, not just a flat list
/// of disclosed bits.
fn public_basis_discussion_e91(results: &Vec<QExecutionResult>) -> PublicDiscussionResult {
    let indexes_to_key = results
        .iter()
        .enumerate()
        .filter(|(_, result)| E91_KEY_PAIRS.contains(&(result.alice_basis, result.bob_basis)))
        .map(|(i, _)| i)
        .collect();

    PublicDiscussionResult {
        alice_public_values: vec![],
        bob_public_values: vec![],
        indexes_to_key,
        results: results.clone(),
    }
}

/// Estimates the correlation `E(alice_basis, bob_basis) = P(agree) - P(disagree)` for
/// every round using that exact basis pair, along with its variance (`Var(E) =
/// (1 - E^2) / n`, since each round contributes an independent +-1-valued sample).
/// Returns `None` if no round used that basis pair.
fn chsh_correlation(results: &[QExecutionResult], pair: (usize, usize)) -> Option<(f64, f64)> {
    let agreements: Vec<bool> = results
        .iter()
        .filter(|result| (result.alice_basis, result.bob_basis) == pair)
        .map(|result| result.alice_value == result.bob_value)
        .collect();

    let n = agreements.len() as f64;
    if n == 0.0 {
        return None;
    }
    let agree_count = agreements.into_iter().filter(|&agree| agree).count() as f64;
    let correlation = (2.0 * agree_count - n) / n; // P(agree) - P(disagree)
    let variance = (1.0 - correlation * correlation) / n;
    Some((correlation, variance))
}

/// E91's security check: computes the CHSH parameter
/// `S = E(0,1) - E(0,2) + E(1,1) + E(1,2)` (see `E91_CHSH_PAIRS`) from the rounds
/// `public_basis_discussion_e91` left out of the key, and checks whether its lower
/// confidence bound still exceeds the classical limit of `2.0` — a one-sided test in
/// the same spirit as `protocol::default_check_security`'s QBER threshold, just for a
/// different statistic. `noise` is not used: unlike the QBER test, this one does not
/// assume a particular noise model, only that a classical (local hidden variable)
/// process cannot produce `|S| > 2.0`.
///
/// Without any noise or eavesdropping, `S` should be close to `2 * sqrt(2) ≈ 2.828`
/// (the Tsirelson bound, the maximum `|S|` quantum mechanics allows) — see
/// `tests/protocols.rs` for the end-to-end check of this.
fn check_security_e91(
    discussion: &PublicDiscussionResult,
    _noise: f64,
    confidence: f64,
) -> (bool, f64) {
    let mut s = 0.0;
    let mut variance_sum = 0.0;
    for (i, &pair) in E91_CHSH_PAIRS.iter().enumerate() {
        match chsh_correlation(&discussion.results, pair) {
            Some((correlation, variance)) => {
                s += if i == 1 { -correlation } else { correlation };
                variance_sum += variance;
            }
            None => return (false, 0.0),
        }
    }

    let normal = Normal::standard();
    let z = normal.inverse_cdf((1.0 + confidence) / 2.0);
    let lower_bound = s.abs() - z * variance_sum.sqrt();

    (lower_bound > 2.0, s.abs())
}

fn public_basis_discussion_b92(results: &Vec<QExecutionResult>) -> PublicDiscussionResult {
    let mut results = results.clone();
    let bob_values: Vec<bool> = results.iter().map(|x| x.bob_value).collect();

    let conclusive_indexes = bob_values
        .iter()
        .enumerate()
        .filter_map(|(i, &value)| if value { Some(i) } else { None })
        .collect::<Vec<usize>>();

    results.iter_mut().enumerate().for_each(|(i, result)| {
        if conclusive_indexes.contains(&i) {
            result.bob_value = result.bob_basis == 0;
        }
        result.alice_value = result.alice_basis == 1;
    });

    let (indexes_to_check, indexes_to_key) = shuffle_and_split(conclusive_indexes);
    let (alice_public_values, bob_public_values) = indexes_to_check
        .iter()
        .map(|&i| (results[i].alice_value, results[i].bob_value))
        .unzip();

    PublicDiscussionResult {
        alice_public_values,
        bob_public_values,
        indexes_to_key,
        results,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rng::set_global_seed;

    #[test]
    fn prepare_bbm92_leaves_bobs_half_matching_alices_value_and_basis() {
        // The crux of how BBM92 reuses the same pipeline as "prepare and send"
        // protocols: once Alice measures her own half, Bob's half must be in
        // exactly the state a directly-prepared qubit would be in for that same
        // value and basis — i.e. measuring it back with the same basis Alice used
        // deterministically reproduces her value.
        set_global_seed(1);
        let bases = vec![I, H];
        for _ in 0..50 {
            let (bobs_qubit, alice_value, alice_basis) = prepare_bbm92(&bases);
            bobs_qubit.apply_local_gate(&bases[alice_basis]);
            assert_eq!(bobs_qubit.measure(), alice_value);
        }
    }

    #[test]
    fn prepare_bbm92_returns_basis_index_within_bounds() {
        set_global_seed(2);
        let bases = vec![I, H];
        for _ in 0..50 {
            let (_, _, basis_id) = prepare_bbm92(&bases);
            assert!(basis_id < bases.len());
        }
    }

    fn make_result(
        alice_value: bool,
        alice_basis: usize,
        bob_value: bool,
        bob_basis: usize,
    ) -> QExecutionResult {
        QExecutionResult::new(alice_value, alice_basis, bob_value, bob_basis, None, None)
    }

    #[test]
    fn chsh_correlation_computes_expected_value_and_variance() {
        let results = vec![
            make_result(true, 0, true, 1),  // agree
            make_result(true, 0, true, 1),  // agree
            make_result(true, 0, false, 1), // disagree
        ];
        let (correlation, variance) = chsh_correlation(&results, (0, 1)).unwrap();
        assert!((correlation - (1.0 / 3.0)).abs() < 1e-9); // 2 agree, 1 disagree -> (2*2-3)/3
        let expected_variance = (1.0 - (1.0 / 3.0_f64).powi(2)) / 3.0;
        assert!((variance - expected_variance).abs() < 1e-9);
    }

    #[test]
    fn chsh_correlation_returns_none_when_basis_pair_unused() {
        let results = vec![make_result(true, 0, true, 0)];
        assert!(chsh_correlation(&results, (1, 2)).is_none());
    }

    #[test]
    fn check_security_e91_detects_violation_from_crafted_results() {
        // Synthetic (not necessarily physically realizable) correlations chosen so
        // the arithmetic can be checked by hand: S = E(0,1) - E(0,2) + E(1,1) + E(1,2)
        // = 1 - (-1) + 1 + 1 = 4, with zero variance (every correlation is exactly
        // +-1), so the test is fully deterministic.
        let mut results = Vec::new();
        results.extend((0..50).map(|_| make_result(true, 0, true, 1))); // (0,1): E=1
        results.extend((0..50).map(|_| make_result(true, 0, false, 2))); // (0,2): E=-1
        results.extend((0..50).map(|_| make_result(true, 1, true, 1))); // (1,1): E=1
        results.extend((0..50).map(|_| make_result(true, 1, true, 2))); // (1,2): E=1

        let discussion = PublicDiscussionResult {
            alice_public_values: vec![],
            bob_public_values: vec![],
            indexes_to_key: vec![],
            results,
        };
        let (secure, s) = check_security_e91(&discussion, 0.0, 0.99);
        assert!(secure);
        assert!((s - 4.0).abs() < 1e-9);
    }

    #[test]
    fn check_security_e91_is_insecure_when_a_chsh_pair_was_never_used() {
        // Only (0,1) ever occurs; (0,2), (1,1) and (1,2) have zero samples.
        let discussion = PublicDiscussionResult {
            alice_public_values: vec![],
            bob_public_values: vec![],
            indexes_to_key: vec![],
            results: vec![make_result(true, 0, true, 1)],
        };
        let (secure, s) = check_security_e91(&discussion, 0.0, 0.99);
        assert!(!secure);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn public_basis_discussion_e91_selects_exactly_the_key_pairs() {
        let results = vec![
            make_result(true, 0, true, 0), // key pair (0,0)
            make_result(true, 2, true, 1), // key pair (2,1)
            make_result(true, 0, true, 1), // CHSH pair (0,1), not key
            make_result(true, 1, true, 0), // neither key nor CHSH, discarded
        ];
        let discussion = public_basis_discussion_e91(&results);
        assert_eq!(discussion.indexes_to_key, vec![0, 1]);
    }
}
