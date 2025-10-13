use crate::types::{ComplexMatrix, Qubit};
use crate::utils::{rand_bool, rand_choose, rand_float, X};
use bon::Builder;

/// Quantum sender entity in a QKD protocol.
///
/// This struct represents Alice's capabilities in the protocol:
/// - Choosing from a set of possible quantum bases.
/// - Preparing qubits in a random state.
/// - Changing the qubit's basis before sending.
#[doc(hidden)]
#[derive(Builder)]
pub struct Sender {
    /// Available quantum bases that Alice can use to prepare and transform qubits.
    pub(crate) posible_basis: Vec<ComplexMatrix>,
    /// Function to randomly change the qubit's basis before sending.
    /// By default, it selects a random basis from `posible_basis` and applies it to the qubit.
    #[builder(default = Box::new(default_change_basis))]
    pub(crate) change_basis: Box<dyn Fn(&mut Qubit, &Vec<ComplexMatrix>) -> usize>,
    /// Function to prepare a qubit in a random state (|0⟩ or |1⟩ with equal probability).
    /// Returns the prepared qubit and its classical bit value.
    #[builder(default = Box::new(default_prepare))]
    pub(crate) prepare: Box<dyn Fn() -> (Qubit, bool)>,
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
    pub(crate) change_basis: Box<dyn Fn(&mut Qubit, &Vec<ComplexMatrix>) -> usize>,
    /// Function to measure a qubit and obtain a classical bit.
    /// The measurement collapses the qubit's state according to its current probabilities.
    #[builder(default = Box::new(default_measure))]
    pub(crate) measure: Box<dyn Fn(&mut Qubit) -> bool>,
    /// Function to attempt restoring a qubit's state after measurement.
    /// Used by Eve to minimize detection during eavesdropping.
    /// By default, it applies the inverse of the basis matrix used for measurement.
    #[builder(default = Box::new(default_try_to_restore_qubit))]
    pub(crate) try_to_restore_qubit: Box<dyn Fn(&mut Qubit, &ComplexMatrix)>,
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
fn default_change_basis(qubit: &mut Qubit, posible_basis: &Vec<ComplexMatrix>) -> usize {
    let (basis_id, matrix) = rand_choose(posible_basis.iter().enumerate().collect());
    qubit.apply_transformation(&matrix);
    basis_id
}

/// Default qubit preparation function for the sender (Alice).
///
/// Prepares a qubit in the |0⟩ state and applies a bit-flip with 50% probability,
/// resulting in either |0⟩ or |1⟩ with equal probability.
///
/// # Returns
///
/// A tuple containing the prepared qubit and its classical bit value (false for |0⟩, true for |1⟩).
fn default_prepare() -> (Qubit, bool) {
    let mut qubit = Qubit::new(); // |0⟩
    let value = rand_bool();
    // Perform a bit-flip with 1/2 probability
    if value {
        qubit.apply_transformation(&X); // |1⟩
    }
    (qubit, value)
}

/// Default qubit measurement function for receivers (Bob/Eve).
///
/// Measures the qubit and collapses its state according to the probability
/// of it being in the |1⟩ state (||one_coef||²).
/// After measurement, the qubit is reset to |0⟩ and transformed to |1⟩ if the
/// measurement result was true.
///
/// # Arguments
///
/// * `qubit` - The qubit to measure.
///
/// # Returns
///
/// The classical bit value obtained from the measurement (false for |0⟩, true for |1⟩).
fn default_measure<'a>(qubit: &'a mut Qubit) -> bool {
    let one_probability = qubit.get_one_coef().norm().powf(2.0);
    qubit.reset(); // |0⟩
    let measurement_result = rand_float() < one_probability;
    if measurement_result {
        qubit.apply_transformation(&X); // |1⟩
    }
    measurement_result
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
fn default_try_to_restore_qubit(qubit: &mut Qubit, basis_matrix: &ComplexMatrix) {
    qubit.apply_transformation(&basis_matrix.invert().unwrap());
}
