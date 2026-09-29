//! Regression tests anchoring the three bugs found and fixed in this crate:
//! 1. Core types not re-exported at the crate root.
//! 2. `NaN` propagation for tiny/empty public-check samples.
//! 3. Opaque panic when a participant's `posible_basis` is empty.

use qkd::{
    build_bb84, set_global_seed, PublicDiscussionResult, QExecutionResult, QKDResult, Receiver,
    Sender, QKD,
};

#[test]
fn core_types_are_usable_from_the_crate_root() {
    // This mainly needs to compile: previously QKD, Sender, Receiver, QExecutionResult,
    // PublicDiscussionResult and QKDResult were only reachable via
    // qkd::protocol::* / qkd::participants::*, contradicting the README's
    // "Build your own protocols" example.
    let alice = Sender::builder()
        .posible_basis(vec![qkd::constants::I])
        .build();
    let bob = Receiver::builder()
        .posible_basis(vec![qkd::constants::I])
        .build();
    let qkd: QKD = QKD::builder()
        .name("RootExportCheck".to_string())
        .alice(alice)
        .bob(bob)
        .build();

    set_global_seed(1);
    let result: QKDResult = qkd.run(100, 0.0, 0.0, 0.99);
    assert!(result.is_considered_secure);

    let _uses_result_types = |_: QExecutionResult, _: PublicDiscussionResult| {};
}

#[test]
fn run_with_zero_qubits_does_not_produce_nan() {
    set_global_seed(2);
    let result = build_bb84().run(0, 0.0, 0.0, 0.9999999999);
    assert!(!result.measured_qber.is_nan());
    assert!(!result.is_considered_secure);
    assert_eq!(result.key_length, None);
}

#[test]
fn run_with_very_few_qubits_does_not_produce_nan() {
    for seed in 0..20 {
        set_global_seed(seed);
        let result = build_bb84().run(2, 0.0, 0.0, 0.9999999999);
        assert!(!result.measured_qber.is_nan());
    }
}

#[test]
#[should_panic(expected = "Alice's posible_basis must not be empty")]
fn empty_alice_basis_panics_with_a_clear_specific_message() {
    let alice = Sender::builder().posible_basis(vec![]).build();
    let bob = Receiver::builder()
        .posible_basis(vec![qkd::constants::I])
        .build();
    let qkd = QKD::builder()
        .name("Bad".to_string())
        .alice(alice)
        .bob(bob)
        .build();
    qkd.run(10, 0.0, 0.0, 0.99);
}
