/// Module containing the implementation of QKD protocol participants (Alice, Bob, and Eve).
/// Provides structs and builders for creating and configuring participants with their
/// respective quantum bases and behaviors.
pub mod participants;

/// Module implementing the core Quantum Key Distribution protocols.
/// Contains the main QKD struct, protocol execution logic, and result types
/// including QKDResult and PublicDiscussionResult.
pub mod protocol;

/// Module defining fundamental quantum types and structures.
/// Includes the Qubit struct and related quantum state representations
/// used throughout the QKD simulations.
pub mod types;

/// Module providing utility functions and common quantum operations.
/// Contains mathematical utilities, basis matrices (I, H, H_Y), and
/// helper functions like shuffle_and_split for protocol execution.
pub mod utils;

use std::collections::HashMap;

use crate::participants::{Receiver, Sender};
use crate::protocol::{PublicDiscussionResult, QExecutionResult, QKDResult, QKD};
use crate::types::Qubit;
use crate::utils::{shuffle_and_split, H, H_Y, I};

/// Builds and configures a QKD instance for the BB84 protocol.
///
/// # Returns
/// A `QKD` instance configured with Alice and Bob using the I and H bases.
pub fn build_bb84() -> QKD {
    let alice = Sender::builder().posible_basis(vec![I, H]).build();
    let bob = Receiver::builder().posible_basis(vec![I, H]).build();

    let bb84 = QKD::builder().alice(alice).bob(bob).build();
    bb84
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

    let six_state = QKD::builder().alice(alice).bob(bob).eve(eve).build();
    six_state
}

/// Builds and configures a QKD instance for the B92 protocol.
///
/// # Returns
/// A `QKD` instance configured with Alice and Bob using the I and H bases,
/// and a custom public basis discussion function for the B92 protocol.
pub fn build_b92() -> QKD {
    let prepare_b92 = Box::new(|| (Qubit::new(), false));

    let alice = Sender::builder()
        .posible_basis(vec![I, H])
        .prepare(prepare_b92)
        .build();
    let bob = Receiver::builder().posible_basis(vec![I, H]).build();

    let b92 = QKD::builder()
        .alice(alice)
        .bob(bob)
        .public_basis_discussion(Box::new(public_basis_discussion_b92))
        .build();

    b92
}

/// Performs the public basis discussion specific to the B92 protocol.
///
/// # Arguments
/// * `results` - Vector of execution results from the B92 protocol.
///
/// # Returns
/// A `PublicDiscussionResult` containing the results of the public discussion phase.
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
            result.bob_value = (1 - result.bob_basis) == 1;
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

/// Returns a map of available QKD protocol configurations.
///
/// # Returns
/// A `HashMap` where the keys are protocol names ("BB84", "SixState", "B92")
/// and the values are pre-configured `QKD` instances for each protocol.
pub fn build_all_available_protocols() -> HashMap<String, QKD> {
    HashMap::from([
        ("BB84".to_string(), build_bb84()),
        ("SixState".to_string(), build_six_state()),
        ("B92".to_string(), build_b92()),
    ])
}

/// Struct representing the result of a single QKD experiment.
///
/// # Fields
/// * `id` - Unique identifier for the experiment.
/// * `protocol_tag` - Name of the protocol used.
/// * `n_qubits` - Number of qubits used in the experiment.
/// * `interception_rate` - Probability that Eve intercepted a qubit (0.0 to 1.0).
/// * `result` - The `QKDResult` containing the detailed results of the experiment.
pub struct ExperimentResult {
    pub id: String,
    pub protocol_tag: String,
    pub n_qubits: usize,
    pub interception_rate: f64,
    pub result: QKDResult,
}

/// Executes a series of QKD experiments across different protocols, qubit counts, and interception rates.
///
/// # Arguments
/// * `protocol` - Vector of protocol names to test.
/// * `number_of_qubits` - Vector of qubit counts to use in each experiment.
/// * `interception_rate` - Vector of interception probabilities to test (0.0 to 1.0).
/// * `repetitions` - Number of times to repeat each combination of parameters.
///
/// # Returns
/// A vector of `ExperimentResult` structs, each representing the result of a single experiment.
pub fn run_experiment(
    protocol: Vec<String>,
    number_of_qubits: Vec<usize>,
    interception_rate: Vec<f64>,
    repetitions: usize,
) -> Vec<ExperimentResult> {
    let protocol_ref = &protocol;
    let number_of_qubits_ref = &number_of_qubits;
    let interception_rate_ref = &interception_rate;
    let repetitions_ref = &repetitions;
    let all_combinations = protocol_ref.iter().flat_map(|protocol_tag| {
        number_of_qubits_ref.iter().flat_map(move |n_qubits| {
            interception_rate_ref
                .iter()
                .flat_map(move |interception_rate| {
                    (0..*repetitions_ref).map(move |repetition| {
                        (protocol_tag, n_qubits, interception_rate, repetition)
                    })
                })
        })
    });

    all_combinations
        .map(
            |(protocol_tag, &n_qubits, &interception_rate, repetition)| {
                let id = format!(
                    "{}_{}_{}-{}",
                    protocol_tag, n_qubits, interception_rate, repetition
                );

                ExperimentResult {
                    id,
                    protocol_tag: protocol_tag.clone(),
                    n_qubits,
                    interception_rate,
                    result: build_all_available_protocols()[protocol_tag]
                        .run(n_qubits, interception_rate),
                }
            },
        )
        .collect()
}
