use crate::participants::{Receiver, Sender};
use crate::utils::{rand_float, suffle_and_split, H, I};
use bon::Builder;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct QExecutionResult {
    pub alice_value: bool,
    pub alice_basis: usize,
    pub bob_value: bool,
    pub bob_basis: usize,
    pub eve_value: Option<bool>,
    pub eve_basis: Option<usize>,
}

impl QExecutionResult {
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

/// The result of the entire QKD protocol.
#[derive(Debug)]
pub struct QKDResult {
    /// Duration of all the quantum communication process.
    pub elapsed_time: Duration,
    /// If the communication is aborted, it is set to false.
    pub is_considered_secure: bool,
    /// Length of the generated key.
    /// If the communication is aborted, it is set to 0.
    pub key_length: Option<usize>,
    /// Quantum Bit Error Rate (QBER) of the final generated key.
    /// If the communication is aborted, it is set to None.
    pub quantum_bit_error_rate: Option<f64>,
    /// The knowledge rate of Eve about the final generated key.
    pub eve_key_knowledge: f64,
}

#[derive(Debug)]
pub struct PublicDiscussionResult {
    pub alice_public_values: Vec<bool>,
    pub bob_public_values: Vec<bool>,
    pub indexes_to_key: Vec<usize>,
    pub results: Vec<QExecutionResult>,
}

#[derive(Builder)]
pub struct QKD {
    alice: Sender,
    bob: Receiver,
    #[builder(default = Receiver::builder().posible_basis(vec![I, H]).build())]
    eve: Receiver,
    #[builder(default = Box::new(default_public_basis_discussion))]
    public_basis_discussion: Box<dyn Fn(&Vec<QExecutionResult>) -> PublicDiscussionResult>,
}

impl QKD {
    pub fn run(&self, number_of_qubits: usize, interception_rate: f64) -> QKDResult {
        let initial_time = Instant::now();
        let results = (0..number_of_qubits)
            .into_iter()
            .map(|_| self.quantum_communication(interception_rate))
            .collect::<Vec<QExecutionResult>>();

        let discussion_result = (self.public_basis_discussion)(&results);
        let results = discussion_result.results;

        let is_considered_secure = self.check_public_values(
            discussion_result.alice_public_values,
            discussion_result.bob_public_values,
        );

        let mut eve_key_knowledge = 0.0;
        let (mut quantum_bit_error_rate, mut key_length) = (None, None);
        if is_considered_secure {
            // TODO: Add privacy amplification and reconciliation techniques
            let ((alice_secret_values, bob_secret_values), eve_secret_values): (
                (Vec<bool>, Vec<bool>),
                Vec<Option<bool>>,
            ) = discussion_result
                .indexes_to_key
                .into_iter()
                .map(|i| {
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
                        acc.1 += if e.map_or(false, |e| a == e) {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    acc
                });

            quantum_bit_error_rate = Some(mismatched_bits / key_length.unwrap() as f64);
            eve_key_knowledge = absolute_eve_knowledge / key_length.unwrap() as f64;
        }
        let elapsed_time = initial_time.elapsed();

        QKDResult {
            elapsed_time,
            is_considered_secure,
            key_length,
            quantum_bit_error_rate,
            eve_key_knowledge,
        }
    }

    fn quantum_communication(&self, interception_rate: f64) -> QExecutionResult {
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

    fn check_public_values(
        &self,
        alice_public_values: Vec<bool>,
        bob_public_values: Vec<bool>,
    ) -> bool {
        alice_public_values
            .into_iter()
            .zip(bob_public_values)
            .all(|(a, b)| a == b)
    }
}

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

    let (indexes_to_check, indexes_to_key) = suffle_and_split(eq_basis_indexes);

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
