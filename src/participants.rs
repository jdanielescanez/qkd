use crate::registry::Qubit;
use crate::rng::{rand_bool, rand_choose};
use crate::types::ComplexMatrix;
use bon::Builder;

/// Quantum sender entity in a QKD protocol.
///
/// This struct represents Alice's capabilities in the protocol: obtaining the qubit
/// that will continue towards Bob (whether by preparing it directly, or by measuring
/// her own half of an entangled pair and letting the other half collapse accordingly),
/// along with her classical bit value and the basis index she used.
#[doc(hidden)]
#[derive(Builder)]
pub struct Sender {
    /// Available quantum bases Alice may use.
    pub(crate) posible_basis: Vec<ComplexMatrix>,
    /// Function that produces the qubit that continues towards Bob, Alice's classical
    /// bit value, and the basis index she used.
    ///
    /// By default (`default_prepare`), this prepares a fresh qubit directly: picks a
    /// random bit, encodes it as |0⟩/|1⟩, then picks and applies a random basis from
    /// `posible_basis`. An entanglement-based protocol can instead provide a `prepare`
    /// that creates an entangled pair, applies a basis to and measures *Alice's own*
    /// half, and returns *Bob's* half (already correctly collapsed by entanglement) —
    /// see `build_bbm92` in `lib.rs`. Fusing "prepare" and "choose a basis" into one
    /// closure (rather than two separate steps, as `Receiver` has for `change_basis`
    /// and `measure`) is what lets both cases share the same `quantum_communication`
    /// pipeline with no branching: a basis choice made *before* measuring Alice's own
    /// qubit cannot be separated from that measurement without corrupting the physics.
    #[builder(default = Box::new(default_prepare))]
    pub(crate) prepare: Box<dyn Fn(&Vec<ComplexMatrix>) -> (Qubit, bool, usize)>,
}

/// Quantum receiver entity in a QKD protocol.
///
/// This struct represents both Bob's and Eve's capabilities:
/// - Choosing from a set of possible quantum bases.
/// - Changing the qubit's basis before measurement.
/// - Measuring the qubit to obtain a classical bit.
/// - Attempting to restore the qubit's state (for Eve).
#[doc(hidden)]
#[derive(Builder)]
pub struct Receiver {
    /// Available quantum bases that the receiver can use to measure qubits.
    pub(crate) posible_basis: Vec<ComplexMatrix>,
    /// Function to randomly change the qubit's basis before measurement.
    /// By default, it selects a random basis from `posible_basis` and applies it to the qubit.
    #[builder(default = Box::new(default_change_basis))]
    pub(crate) change_basis: Box<dyn Fn(&Qubit, &Vec<ComplexMatrix>) -> usize>,
    /// Function to measure a qubit and obtain a classical bit.
    /// The measurement collapses the qubit's state according to its current probabilities.
    #[builder(default = Box::new(default_measure))]
    pub(crate) measure: Box<dyn Fn(&Qubit) -> bool>,
    /// Function to attempt restoring a qubit's state after measurement.
    /// Used by Eve to minimize detection during eavesdropping.
    /// By default, it applies the inverse of the basis matrix used for measurement.
    #[builder(default = Box::new(default_try_to_restore_qubit))]
    pub(crate) try_to_restore_qubit: Box<dyn Fn(&Qubit, &ComplexMatrix)>,
}

/// Default basis change function for quantum entities.
///
/// Randomly selects a basis from the available options and applies it to the qubit.
/// Returns the index of the selected basis.
///
/// # Arguments
///
/// * `qubit` - The qubit to transform.
/// * `posible_basis` - Available quantum bases to choose from.
///
/// # Returns
///
/// The index of the selected basis in the `posible_basis` vector.
fn default_change_basis(qubit: &Qubit, posible_basis: &Vec<ComplexMatrix>) -> usize {
    let (basis_id, matrix) = rand_choose(posible_basis.iter().enumerate().collect());
    qubit.apply_local_gate(matrix);
    basis_id
}

/// Default qubit preparation function for the sender (Alice).
///
/// Prepares a qubit in a computational basis state (|0⟩ or |1⟩) chosen with equal
/// probability, then picks and applies a random basis from `posible_basis`.
///
/// # Arguments
///
/// * `posible_basis` - Available quantum bases to choose from.
///
/// # Returns
///
/// A tuple of the prepared qubit, its classical bit value (false for |0⟩, true for
/// |1⟩), and the index of the basis applied to it.
fn default_prepare(posible_basis: &Vec<ComplexMatrix>) -> (Qubit, bool, usize) {
    let value = rand_bool();
    let qubit = Qubit::create_basis_state(value);
    let (basis_id, matrix) = rand_choose(posible_basis.iter().enumerate().collect());
    qubit.apply_local_gate(matrix);
    (qubit, value, basis_id)
}

/// Default qubit measurement function for receivers (Bob/Eve).
///
/// Measures the qubit in the computational basis, collapsing (and renormalizing) the
/// quantum state it belongs to.
///
/// # Arguments
///
/// * `qubit` - The qubit to measure.
///
/// # Returns
///
/// The classical bit value obtained from the measurement (false for |0⟩, true for |1⟩).
fn default_measure(qubit: &Qubit) -> bool {
    qubit.measure()
}

/// Default qubit restoration function for eavesdroppers (Eve).
///
/// Attempts to restore a qubit's state after measurement by applying
/// the inverse of the basis matrix used during the measurement.
///
/// # Arguments
///
/// * `qubit` - The qubit to restore.
/// * `basis_matrix` - The basis matrix that was used for measurement.
fn default_try_to_restore_qubit(qubit: &Qubit, basis_matrix: &ComplexMatrix) {
    qubit.apply_local_gate(&basis_matrix.invert().unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{H, I};
    use crate::rng::set_global_seed;

    #[test]
    fn default_prepare_basis_matches_the_value_when_measured_back_in_the_same_basis() {
        set_global_seed(1);
        let bases = vec![I, H];
        for _ in 0..50 {
            let (qubit, value, basis_id) = default_prepare(&bases);
            // Measuring with the same basis Alice used must deterministically
            // reproduce her value (bases are self-inverse here: I and H).
            qubit.apply_local_gate(&bases[basis_id]);
            assert_eq!(qubit.measure(), value);
        }
    }

    #[test]
    fn default_prepare_returns_basis_index_within_bounds() {
        set_global_seed(2);
        let bases = vec![I, H];
        for _ in 0..50 {
            let (_, _, basis_id) = default_prepare(&bases);
            assert!(basis_id < bases.len());
        }
    }

    #[test]
    fn default_prepare_produces_both_values_over_many_runs() {
        set_global_seed(3);
        let bases = vec![I];
        let values: Vec<bool> = (0..200).map(|_| default_prepare(&bases).1).collect();
        assert!(values.iter().any(|&v| v));
        assert!(values.iter().any(|&v| !v));
    }

    #[test]
    fn default_change_basis_returns_index_within_bounds() {
        set_global_seed(3);
        let bases = vec![I, H];
        for _ in 0..50 {
            let qubit = Qubit::create_basis_state(false);
            let idx = default_change_basis(&qubit, &bases);
            assert!(idx < bases.len());
        }
    }

    #[test]
    fn default_change_basis_with_single_basis_is_deterministic() {
        set_global_seed(4);
        let bases = vec![I];
        let qubit = Qubit::create_basis_state(false);
        assert_eq!(default_change_basis(&qubit, &bases), 0);
    }

    #[test]
    fn measuring_zero_state_always_returns_false() {
        set_global_seed(5);
        for _ in 0..50 {
            let qubit = Qubit::create_basis_state(false);
            assert!(!default_measure(&qubit));
        }
    }

    #[test]
    fn measuring_one_state_always_returns_true() {
        set_global_seed(6);
        for _ in 0..50 {
            let qubit = Qubit::create_basis_state(true);
            assert!(default_measure(&qubit));
        }
    }

    #[test]
    fn measure_collapses_qubit_to_a_deterministic_outcome() {
        set_global_seed(7);
        let qubit = Qubit::create_basis_state(false);
        qubit.apply_local_gate(&H);
        let result = default_measure(&qubit);
        // Once collapsed, measuring again must deterministically reproduce the same
        // outcome (there is no amplitude introspection on `Qubit` to check directly).
        assert_eq!(qubit.measure(), result);
    }

    #[test]
    fn try_to_restore_qubit_undoes_a_basis_change() {
        // Without a fixed seed (and enough repetitions), a no-op "restore" would still
        // pass about half the time by sheer luck, since the un-restored |+⟩ state
        // measures to `false` with 50% probability on its own.
        set_global_seed(8);
        for _ in 0..30 {
            let qubit = Qubit::create_basis_state(false);
            qubit.apply_local_gate(&H);
            default_try_to_restore_qubit(&qubit, &H);
            assert!(!qubit.measure());
        }
    }
}
