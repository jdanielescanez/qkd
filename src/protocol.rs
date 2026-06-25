use crate::constants::{H, I, X};
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

    /// Quantum Bit Error Rate (QBER) of the public values.
    pub measured_qber: f64,

    /// Estimated fraction of the final key known by an eavesdropper (Eve).
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
        let initial_time = Instant::now();
        let results = (0..number_of_qubits)
            .map(|_| self.quantum_communication(interception_rate, noise))
            .collect::<Vec<QExecutionResult>>();

        let discussion_result = (self.public_basis_discussion)(&results);
        let results = discussion_result.results;

        let (is_considered_secure, measured_qber) = self.check_public_values(
            discussion_result.alice_public_values,
            discussion_result.bob_public_values,
            noise,
            confidence,
        );

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
        // Alice
        let (mut qubit, alice_value) = (self.alice.prepare)();
        let alice_basis = (self.alice.change_basis)(&mut qubit, &self.alice.posible_basis);

        // Eve
        let mut eve_basis = None;
        let mut eve_value = None;
        if rand_float() < interception_rate {
            eve_basis = Some((self.eve.change_basis)(&mut qubit, &self.eve.posible_basis));
            eve_value = Some((self.eve.measure)(&mut qubit));
            (self.eve.try_to_restore_qubit)(
                &mut qubit,
                &self.eve.posible_basis[eve_basis.unwrap()],
            );
        }

        // Perform bit-flip
        if rand_float() < noise {
            qubit.apply_transformation(&X);
        }

        // Bob
        let bob_basis = (self.bob.change_basis)(&mut qubit, &self.bob.posible_basis);
        let bob_value = (self.bob.measure)(&mut qubit);

        QExecutionResult::new(
            alice_value,
            alice_basis,
            bob_value,
            bob_basis,
            eve_value,
            eve_basis,
        )
    }

    /// Checks if the public values announced by Alice and Bob match.
    ///
    /// # Arguments
    ///
    /// * `alice_public_values` - Public values announced by Alice.
    /// * `bob_public_values` - Public values announced by Bob.
    ///
    /// # Returns
    ///
    /// `true` if all public values match, indicating no eavesdropping was detected.
    fn check_public_values(
        &self,
        alice_public_values: Vec<bool>,
        bob_public_values: Vec<bool>,
        noise: f64,
        confidence: f64,
    ) -> (bool, f64) {
        let number_of_qubits = alice_public_values.len() as f64;
        let normal = Normal::standard();

        let p = (1.0 + confidence) / 2.0;
        let z = normal.inverse_cdf(p);

        let threshold = noise + z / number_of_qubits.sqrt() * (noise * (1.0 - noise)).sqrt();

        let measured_qber = alice_public_values
            .into_iter()
            .zip(bob_public_values)
            .filter(|(a, b)| a != b)
            .count() as f64
            / number_of_qubits;

        (measured_qber <= threshold, measured_qber)
    }
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
