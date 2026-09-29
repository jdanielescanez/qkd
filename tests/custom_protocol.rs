//! Tests that build custom QKD protocols directly through the public builder API
//! (`QKD`, `Sender`, `Receiver`, `QExecutionResult`, `PublicDiscussionResult`), the way
//! the README's "Build your own protocols" section describes. These also act as a
//! regression test for making sure these types are re-exported at the crate root.

use qkd::constants::{H, H_Y, I};
use qkd::{
    set_global_seed, PublicDiscussionResult, QExecutionResult, QKD,
};
use qkd::{Qubit, Receiver, Sender};
use std::cell::Cell;

#[test]
fn custom_six_state_replica_runs_like_the_built_in_one() {
    // Same definition as `build_six_state()`, but constructed here using only the
    // public API, as shown in the README.
    let alice = Sender::builder().posible_basis(vec![I, H, H_Y]).build();
    let bob = Receiver::builder()
        .posible_basis(vec![I, H, H_Y.invert().unwrap()])
        .build();
    let eve = Receiver::builder()
        .posible_basis(vec![I, H, H_Y.invert().unwrap()])
        .build();

    let qkd = QKD::builder()
        .name("CustomSixState".to_string())
        .alice(alice)
        .bob(bob)
        .eve(eve)
        .build();

    set_global_seed(42);
    let result = qkd.run(5000, 0.0, 0.0, 0.9999999999);
    assert_eq!(qkd.get_name(), "CustomSixState");
    assert!(result.is_considered_secure);
    assert_eq!(result.final_key_qber, Some(0.0));
}

/// Fully deterministic scenario (no reliance on any RNG outcome) that pins down the
/// documented `eve_knowledge` "lower bound" semantics: Eve intercepts every round and
/// always measures the correct bit (single shared I basis, no channel noise), yet one
/// key bit is deliberately made to mismatch between Alice and Bob. That mismatched bit
/// is excluded from `eve_knowledge`'s numerator even though Eve did know it, so the
/// reported `eve_knowledge` (0.75) undercounts Eve's true knowledge (which is 100% of
/// the 4 key bits).
#[test]
fn eve_knowledge_is_a_lower_bound_when_the_final_key_has_errors() {
    let alice_prepare = Box::new(|| {
        let mut qubit = Qubit::new();
        qubit.apply_transformation(&qkd::constants::X); // |1⟩, matching the returned `true`
        (qubit, true)
    }) as Box<dyn Fn() -> (Qubit, bool)>;

    let bob_round = Cell::new(0usize);
    let bob_measure = Box::new(move |qubit: &mut Qubit| {
        let round = bob_round.get();
        bob_round.set(round + 1);
        let true_value = qubit.get_one_coef().norm() > 0.5;
        let reported_value = if round == 2 { !true_value } else { true_value };
        qubit.reset();
        if reported_value {
            qubit.apply_transformation(&qkd::constants::X);
        }
        reported_value
    }) as Box<dyn Fn(&mut Qubit) -> bool>;

    let alice = Sender::builder()
        .posible_basis(vec![I])
        .prepare(alice_prepare)
        .build();
    let bob = Receiver::builder()
        .posible_basis(vec![I])
        .measure(bob_measure)
        .build();
    let eve = Receiver::builder().posible_basis(vec![I]).build();

    // Deterministic split: odd rounds go to the public check (all matching, so the
    // security check passes even with noise = 0.0), even rounds go to the final key
    // (containing the single forced mismatch at round 2).
    let discussion = Box::new(|results: &Vec<QExecutionResult>| {
        let indexes_to_check: Vec<usize> = (0..results.len()).filter(|i| i % 2 == 1).collect();
        let indexes_to_key: Vec<usize> = (0..results.len()).filter(|i| i % 2 == 0).collect();
        let (alice_public_values, bob_public_values) = indexes_to_check
            .iter()
            .map(|&i| (results[i].alice_value, results[i].bob_value))
            .unzip();
        PublicDiscussionResult {
            alice_public_values,
            bob_public_values,
            indexes_to_key,
            results: results.clone(),
        }
    });

    let qkd = QKD::builder()
        .name("Deterministic".to_string())
        .alice(alice)
        .bob(bob)
        .eve(eve)
        .public_basis_discussion(discussion)
        .build();

    let result = qkd.run(8, 1.0, 0.0, 0.9999999999);

    assert!(result.is_considered_secure);
    assert_eq!(result.key_length, Some(4));
    assert_eq!(result.final_key_qber, Some(0.25));
    // Eve actually knew all 4 key bits correctly, but only 3 are credited to her
    // because round 2 is a mismatch between Alice and Bob.
    assert_eq!(result.eve_knowledge, 0.75);
}
