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
mod types;

/// Module containing fundamental quantum constant matrices.
/// Provides predefined quantum gates and operations used in QKD protocols,
/// including identity (I), Hadamard (H), Pauli-X (X), and Y-basis Hadamard (H_Y) matrices.
pub mod constants;

/// Module providing utility functions and common quantum operations.
/// Contains mathematical utilities, basis matrices (I, H, H_Y), and
/// helper functions like shuffle_and_split for protocol execution.
pub mod helpers;

use constants::{H, H_Y, I};
use helpers::shuffle_and_split;
use participants::{Receiver, Sender};
use protocol::{PublicDiscussionResult, QExecutionResult, QKD};
pub use types::{ComplexMatrix, Qubit};

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
    let prepare_b92 = Box::new(|| (Qubit::new(), false));

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
