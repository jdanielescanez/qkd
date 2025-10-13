use crate::types::ComplexMatrix;

use num_complex::Complex64;
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use rand::{rng, Rng};
use std::f64::consts::SQRT_2;

pub fn rand_choose<T: Clone>(vec: Vec<T>) -> T {
    let mut rng = rng();
    vec.choose(&mut rng).cloned().expect("Vec cannot be empty")
}

pub fn rand_bool() -> bool {
    let mut rng = rng();
    *[true, false].choose(&mut rng).unwrap()
}

pub fn rand_float() -> f64 {
    rng().random()
}

pub fn suffle_and_split<T>(mut vector: Vec<T>) -> (Vec<T>, Vec<T>)
where
    T: Clone,
{
    let mut rng = rng();
    vector.shuffle(&mut rng);

    let half = vector.len() / 2;
    let first_half = vector[..half].to_vec();
    let second_half = vector[half..].to_vec();
    (first_half, second_half)
}

pub const I: ComplexMatrix = ComplexMatrix([
    [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
    [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
]);

pub const H: ComplexMatrix = ComplexMatrix([
    [
        Complex64::new(1.0 / SQRT_2, 0.0),
        Complex64::new(1.0 / SQRT_2, 0.0),
    ],
    [
        Complex64::new(1.0 / SQRT_2, 0.0),
        Complex64::new(-1.0 / SQRT_2, 0.0),
    ],
]);

pub const X: ComplexMatrix = ComplexMatrix([
    [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
    [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
]);

pub const H_Y: ComplexMatrix = ComplexMatrix([
    [Complex64::new(1.0, 0.0), Complex64::new(1.0, 0.0)],
    [Complex64::new(0.0, 1.0), Complex64::new(0.0, -1.0)],
]);
