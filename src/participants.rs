use bon::Builder;

use crate::types::{ComplexMatrix, Qubit};
use crate::utils::{rand_bool, rand_choose, rand_float, X};

#[derive(Builder)]
pub struct Sender {
    pub(crate) posible_basis: Vec<ComplexMatrix>,
    #[builder(default = Box::new(default_change_basis))]
    pub(crate) change_basis: Box<dyn Fn(&mut Qubit, &Vec<ComplexMatrix>) -> usize>,
    #[builder(default = Box::new(default_prepare))]
    pub(crate) prepare: Box<dyn Fn() -> (Qubit, bool)>,
}

#[derive(Builder)]
pub struct Receiver {
    pub(crate) posible_basis: Vec<ComplexMatrix>,
    #[builder(default = Box::new(default_change_basis))]
    pub(crate) change_basis: Box<dyn Fn(&mut Qubit, &Vec<ComplexMatrix>) -> usize>,
    #[builder(default = Box::new(default_measure))]
    pub(crate) measure: Box<dyn Fn(&mut Qubit) -> bool>,
    #[builder(default = Box::new(default_try_to_restore_qubit))]
    pub(crate) try_to_restore_qubit: Box<dyn Fn(&mut Qubit, &ComplexMatrix)>,
}

fn default_change_basis(qubit: &mut Qubit, posible_basis: &Vec<ComplexMatrix>) -> usize {
    let (basis_id, matrix) = rand_choose(posible_basis.iter().enumerate().collect());
    qubit.apply_transformation(&matrix);
    basis_id
}

fn default_prepare() -> (Qubit, bool) {
    let mut qubit = Qubit::new(); // |0>
    let value = rand_bool();
    // Perform a bit-flip with 1/2 probability
    if value {
        qubit.apply_transformation(&X); // |1>
    }
    (qubit, value)
}

// Perform a bit-flip with ||one_coef||^2 probability
fn default_measure<'a>(qubit: &'a mut Qubit) -> bool {
    let one_probability = qubit.get_one_coef().norm().powf(2.0);
    qubit.reset(); // |0>
    let measurement_result = rand_float() < one_probability;
    if measurement_result {
        qubit.apply_transformation(&X); // |1>
    }
    measurement_result
}

fn default_try_to_restore_qubit(qubit: &mut Qubit, basis_matrix: &ComplexMatrix) {
    qubit.apply_transformation(&basis_matrix.invert().unwrap());
}
