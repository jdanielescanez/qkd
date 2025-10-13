use crate::types::ComplexMatrix;
use num_complex::Complex64;
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use rand::{rng, Rng};
use std::f64::consts::SQRT_2;

/// Randomly selects an element from a vector.
///
/// # Arguments
///
/// * `vec` - A non-empty vector of elements to choose from.
///
/// # Returns
///
/// A randomly selected element from the vector.
///
/// # Panics
///
/// Panics if the input vector is empty.
pub fn rand_choose<T: Clone>(vec: Vec<T>) -> T {
    let mut rng = rng();
    vec.choose(&mut rng).cloned().expect("Vec cannot be empty")
}

/// Generates a random boolean value.
///
/// # Returns
///
/// `true` or `false` with equal probability (50% each).
pub fn rand_bool() -> bool {
    let mut rng = rng();
    *[true, false].choose(&mut rng).unwrap()
}

/// Generates a random floating-point number in the range [0, 1).
///
/// # Returns
///
/// A random `f64` value uniformly distributed in the interval [0, 1).
pub fn rand_float() -> f64 {
    rng().random()
}

/// Randomly shuffles a vector and splits it into two halves.
///
/// # Arguments
///
/// * `vector` - The vector to shuffle and split.
///
/// # Returns
///
/// A tuple containing two new vectors:
/// - The first half of the shuffled vector.
/// - The second half of the shuffled vector.
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

/// Identity matrix (I) for quantum operations.
///
/// Represents the quantum identity operation that leaves qubits unchanged.
/// Mathematically equivalent to:
/// ```text
/// | 1  0 |
/// | 0  1 |
/// ```
pub const I: ComplexMatrix = ComplexMatrix([
    [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
    [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
]);

/// Hadamard matrix (H) for quantum operations.
///
/// Represents the quantum Hadamard gate that creates superposition states.
/// Mathematically equivalent to:
/// ```text
/// | 1/√2   1/√2 |
/// | 1/√2  -1/√2 |
/// ```
/// Transforms |0⟩ to (|0⟩ + |1⟩)/√2 and |1⟩ to (|0⟩ - |1⟩)/√2.
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

/// Pauli-X matrix (X) for quantum operations.
///
/// Represents the quantum NOT gate that flips qubit states.
/// Mathematically equivalent to:
/// ```text
/// | 0  1 |
/// | 1  0 |
/// ```
/// Transforms |0⟩ to |1⟩ and |1⟩ to |0⟩.
pub const X: ComplexMatrix = ComplexMatrix([
    [Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
    [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
]);

/// Y-basis Hadamard quantum gate.
///
/// Analogous to the standard Hadamard gate (H), which transforms between the
/// Z-basis and X-basis, this gate transforms between the Z-basis and Y-basis.
///
/// Mathematically represented as:
/// ```text
/// | 1/√2    1/√2 |
/// | i/√2   -i/√2 |
/// ```
/// where i is the imaginary unit (√-1).
///
/// This gate performs the following basis transformations:
/// - |0⟩ → |+i⟩ = (|0⟩ + i|1⟩)/√2
/// - |1⟩ → |-i⟩ = (|0⟩ - i|1⟩)/√2
pub const H_Y: ComplexMatrix = ComplexMatrix([
    [
        Complex64::new(1.0 / SQRT_2, 0.0),
        Complex64::new(1.0 / SQRT_2, 0.0),
    ],
    [
        Complex64::new(0.0, 1.0 / SQRT_2),
        Complex64::new(0.0, -1.0 / SQRT_2),
    ],
]);
