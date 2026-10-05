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

use constants::{H, H_Y, I};
pub use participants::{Receiver, Sender};
pub use protocol::{PublicDiscussionResult, QExecutionResult, QKDResult, QKD};
pub use registry::Qubit;
pub use rng::set_global_seed;
use rng::{rand_choose, shuffle_and_split};
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
}
