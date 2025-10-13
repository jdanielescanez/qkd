pub mod participants;
pub mod protocol;
pub mod types;
pub mod utils;

use crate::participants::{Receiver, Sender};
use crate::protocol::{PublicDiscussionResult, QExecutionResult, QKDResult, QKD};
use crate::types::Qubit;
use crate::utils::{suffle_and_split, H, H_Y, I};

pub fn run_bb84(number_of_qubits: usize, interception_rate: f64) -> QKDResult {
    let alice = Sender::builder().posible_basis(vec![I, H]).build();
    let bob = Receiver::builder().posible_basis(vec![I, H]).build();

    let bb84 = QKD::builder().alice(alice).bob(bob).build();
    bb84.run(number_of_qubits, interception_rate)
}

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

    let (indexes_to_check, indexes_to_key) = suffle_and_split(conclusive_indexes);
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
