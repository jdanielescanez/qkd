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

use crate::participants::{Receiver, Sender};
use crate::protocol::{PublicDiscussionResult, QExecutionResult, QKDResult, QKD};
use crate::types::Qubit;
use crate::utils::{shuffle_and_split, H, H_Y, I};

/// Executes the BB84 QKD protocol with the specified number of qubits and interception rate.
///
/// # Arguments
/// * `number_of_qubits` - Number of qubits to be used in the protocol.
/// * `interception_rate` - Probability that Eve intercepts a qubit (0.0 to 1.0).
///
/// # Returns
/// A `QKDResult` containing the protocol execution results.
pub fn run_bb84(number_of_qubits: usize, interception_rate: f64) -> QKDResult {
    let alice = Sender::builder().posible_basis(vec![I, H]).build();
    let bob = Receiver::builder().posible_basis(vec![I, H]).build();

    let bb84 = QKD::builder().alice(alice).bob(bob).build();
    bb84.run(number_of_qubits, interception_rate)
}

/// Executes the Six-State QKD protocol with the specified number of qubits and interception rate.
///
/// # Arguments
/// * `number_of_qubits` - Number of qubits to be used in the protocol.
/// * `interception_rate` - Probability that Eve intercepts a qubit (0.0 to 1.0).
///
/// # Returns
/// A `QKDResult` containing the protocol execution results.
pub fn run_six_state(number_of_qubits: usize, interception_rate: f64) -> QKDResult {
    let alice = Sender::builder().posible_basis(vec![I, H, H_Y]).build();
    let bob = Receiver::builder()
        .posible_basis(vec![I, H, H_Y.invert().unwrap()])
        .build();
    let eve = Receiver::builder()
        .posible_basis(vec![I, H, H_Y.invert().unwrap()])
        .build();

    let six_state = QKD::builder().alice(alice).bob(bob).eve(eve).build();
    six_state.run(number_of_qubits, interception_rate)
}

/// Executes the B92 QKD protocol with the specified number of qubits and interception rate.
///
/// # Arguments
/// * `number_of_qubits` - Number of qubits to be used in the protocol.
/// * `interception_rate` - Probability that Eve intercepts a qubit (0.0 to 1.0).
///
/// # Returns
/// A `QKDResult` containing the protocol execution results.
pub fn run_b92(number_of_qubits: usize, interception_rate: f64) -> QKDResult {
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
    b92.run(number_of_qubits, interception_rate)
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
