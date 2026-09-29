//! End-to-end behavior tests for the built-in protocols, using only the public API.

use qkd::{build_b92, build_bb84, build_six_state, set_global_seed};

#[test]
fn bb84_noiseless_no_eavesdropper_is_perfectly_secure() {
    set_global_seed(100);
    let result = build_bb84().run(5000, 0.0, 0.0, 0.9999999999);
    assert!(result.is_considered_secure);
    assert_eq!(result.measured_qber, 0.0);
    assert_eq!(result.final_key_qber, Some(0.0));
    assert!(result.key_length.unwrap() > 0);
}

#[test]
fn six_state_noiseless_no_eavesdropper_is_perfectly_secure() {
    set_global_seed(101);
    let result = build_six_state().run(5000, 0.0, 0.0, 0.9999999999);
    assert!(result.is_considered_secure);
    assert_eq!(result.final_key_qber, Some(0.0));
    assert!(result.key_length.unwrap() > 0);
}

#[test]
fn b92_noiseless_no_eavesdropper_is_perfectly_secure() {
    set_global_seed(102);
    let result = build_b92().run(5000, 0.0, 0.0, 0.9999999999);
    assert!(result.is_considered_secure);
    assert_eq!(result.final_key_qber, Some(0.0));
    assert!(result.key_length.unwrap() > 0);
}

#[test]
fn bb84_full_interception_is_detected() {
    set_global_seed(200);
    let result = build_bb84().run(5000, 1.0, 0.0, 0.9999999999);
    assert!(!result.is_considered_secure);
    // Eve measuring in a random basis on every round introduces roughly 25% QBER
    // on the publicly disclosed bits (well above any reasonable noise floor).
    assert!(result.measured_qber > 0.1);
}

#[test]
fn bb84_full_interception_key_metrics_are_unset_when_insecure() {
    set_global_seed(201);
    let result = build_bb84().run(5000, 1.0, 0.0, 0.9999999999);
    assert!(!result.is_considered_secure);
    assert_eq!(result.key_length, None);
    assert_eq!(result.final_key_qber, None);
    // eve_knowledge is left at its default (not computed) when the run is aborted.
    assert_eq!(result.eve_knowledge, 0.0);
}

#[test]
fn six_state_full_interception_is_detected() {
    set_global_seed(202);
    let result = build_six_state().run(5000, 1.0, 0.0, 0.9999999999);
    assert!(!result.is_considered_secure);
}

#[test]
fn declared_noise_without_eavesdropper_is_accepted_even_when_high() {
    set_global_seed(300);
    // With no eavesdropper, the observed error rate centers on the declared `noise`
    // itself, so the security check (calibrated around that same value) should accept
    // it even when `noise` is high.
    let result = build_bb84().run(5000, 0.0, 0.3, 0.999999);
    assert!(result.is_considered_secure);
}

#[test]
fn declared_noise_matches_the_observed_error_rate() {
    set_global_seed(302);
    // Not just "still secure": the actually-applied bit-flip noise should make the
    // publicly observed QBER track the declared `noise`, up to a known factor. This
    // would catch a broken noise application (e.g. the bit-flip effectively never
    // triggering), which a bare `is_considered_secure` check would miss since "no
    // noise applied" also happens to pass the security check.
    //
    // The noise step applies an X gate to the qubit in whatever basis Alice encoded
    // it in. X leaves H-basis states (|+>, |->) unchanged up to a global phase (they
    // are eigenstates of X), so it only produces an observable error for the I-basis
    // half of BB84's [I, H] encoding. Over the publicly checked (basis-matching)
    // rounds, this halves the expected error rate relative to the declared `noise`.
    let noise = 0.3;
    let result = build_bb84().run(20000, 0.0, noise, 0.999999);
    assert!(result.is_considered_secure);
    assert!((result.measured_qber - noise / 2.0).abs() < 0.03);
}

#[test]
fn small_declared_noise_is_accepted_within_confidence() {
    set_global_seed(301);
    let result = build_bb84().run(5000, 0.0, 0.02, 0.999);
    assert!(result.is_considered_secure);
}

#[test]
fn protocol_names_are_set_correctly() {
    assert_eq!(build_bb84().get_name(), "BB84");
    assert_eq!(build_six_state().get_name(), "SixState");
    assert_eq!(build_b92().get_name(), "B92");
}
