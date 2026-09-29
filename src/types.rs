use num_complex::Complex64;
use std::ops::{Add, Div};

// TODO: Use a standard library for matrices.
/// Represents a 2x2 matrix of complex numbers.
#[derive(Clone, Copy)]
pub struct ComplexMatrix(pub [[Complex64; 2]; 2]);

impl ComplexMatrix {
    /// Computes the inverse of the matrix if it exists.
    /// Returns `None` if the matrix is not invertible (determinant is zero).
    pub fn invert(&self) -> Option<ComplexMatrix> {
        let a = self.0[0][0];
        let b = self.0[0][1];
        let c = self.0[1][0];
        let d = self.0[1][1];

        let det = a * d - b * c;
        if det == Complex64::new(0.0, 0.0) {
            return None; // Matrix is not invertible
        }

        let inv_det = Complex64::new(1.0, 0.0) / det;
        Some(ComplexMatrix([
            [d * inv_det, -b * inv_det],
            [-c * inv_det, a * inv_det],
        ]))
    }
}

/// Implements matrix addition for `ComplexMatrix`.
impl Add<ComplexMatrix> for ComplexMatrix {
    type Output = Self;
    fn add(self, matrix: Self) -> Self::Output {
        ComplexMatrix([
            [self.0[0][0] + matrix.0[0][0], self.0[0][1] + matrix.0[0][1]],
            [self.0[1][0] + matrix.0[1][0], self.0[1][1] + matrix.0[1][1]],
        ])
    }
}

/// Allows conversion from a 2x2 array of `Complex64` to `ComplexMatrix`.
impl Into<ComplexMatrix> for [[Complex64; 2]; 2] {
    fn into(self) -> ComplexMatrix {
        ComplexMatrix(self)
    }
}

/// Implements scalar division for `ComplexMatrix`.
impl Div<f64> for ComplexMatrix {
    type Output = Self;
    fn div(self, divisor: f64) -> Self::Output {
        ComplexMatrix([
            [self.0[0][0] / divisor, self.0[0][1] / divisor],
            [self.0[1][0] / divisor, self.0[1][1] / divisor],
        ])
    }
}

/// Represents a qubit with a quantum state as a linear combination of |0⟩ and |1⟩.
pub struct Qubit {
    state: (Complex64, Complex64),
}

impl Qubit {
    /// Creates a new qubit in the |0⟩ state.
    pub fn new() -> Self {
        Qubit {
            state: (Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)),
        }
    }

    /// Resets the qubit to the |0⟩ state.
    pub fn reset(&mut self) {
        *self = Qubit::new();
    }

    /// Applies a quantum transformation (unitary matrix) to the qubit.
    pub fn apply_transformation(&mut self, matrix: &ComplexMatrix) {
        self.state = (
            self.state.0 * matrix.0[0][0] + self.state.1 * matrix.0[0][1],
            self.state.0 * matrix.0[1][0] + self.state.1 * matrix.0[1][1],
        );
    }

    /// Returns the coefficient for the |0⟩ state.
    pub fn get_zero_coef(&self) -> Complex64 {
        self.state.0
    }

    /// Returns the coefficient for the |1⟩ state.
    pub fn get_one_coef(&self) -> Complex64 {
        self.state.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{H, I, X};
    use std::f64::consts::SQRT_2;

    const EPS: f64 = 1e-9;

    fn assert_complex_eq(actual: Complex64, expected: Complex64) {
        assert!(
            (actual - expected).norm() < EPS,
            "expected {expected:?}, got {actual:?}"
        );
    }

    #[test]
    fn invert_identity_returns_identity() {
        let inv = I.invert().expect("I must be invertible");
        assert_complex_eq(inv.0[0][0], Complex64::new(1.0, 0.0));
        assert_complex_eq(inv.0[0][1], Complex64::new(0.0, 0.0));
        assert_complex_eq(inv.0[1][0], Complex64::new(0.0, 0.0));
        assert_complex_eq(inv.0[1][1], Complex64::new(1.0, 0.0));
    }

    #[test]
    fn invert_singular_matrix_returns_none() {
        // Rows are proportional -> determinant is zero -> not invertible.
        let singular = ComplexMatrix([
            [Complex64::new(1.0, 0.0), Complex64::new(2.0, 0.0)],
            [Complex64::new(2.0, 0.0), Complex64::new(4.0, 0.0)],
        ]);
        assert!(singular.invert().is_none());
    }

    #[test]
    fn add_sums_matrices_entrywise() {
        // H has non-zero off-diagonal entries, unlike I: summing I with itself leaves
        // the off-diagonal at 0 regardless of whether it is added, subtracted or
        // multiplied, so it can't distinguish those operators from one another.
        let sum = H + H;
        assert_complex_eq(sum.0[0][0], Complex64::new(2.0 / SQRT_2, 0.0));
        assert_complex_eq(sum.0[0][1], Complex64::new(2.0 / SQRT_2, 0.0));
        assert_complex_eq(sum.0[1][0], Complex64::new(2.0 / SQRT_2, 0.0));
        assert_complex_eq(sum.0[1][1], Complex64::new(-2.0 / SQRT_2, 0.0));
    }

    #[test]
    fn div_scales_matrix_entrywise() {
        // Same rationale as above: use H (non-zero off-diagonal) so every entry,
        // including the off-diagonal ones, is actually exercised.
        let halved = (H + H) / 2.0;
        assert_complex_eq(halved.0[0][0], H.0[0][0]);
        assert_complex_eq(halved.0[0][1], H.0[0][1]);
        assert_complex_eq(halved.0[1][0], H.0[1][0]);
        assert_complex_eq(halved.0[1][1], H.0[1][1]);
    }

    #[test]
    fn new_qubit_starts_in_zero_state() {
        let qubit = Qubit::new();
        assert_complex_eq(qubit.get_zero_coef(), Complex64::new(1.0, 0.0));
        assert_complex_eq(qubit.get_one_coef(), Complex64::new(0.0, 0.0));
    }

    #[test]
    fn x_gate_flips_zero_to_one() {
        let mut qubit = Qubit::new();
        qubit.apply_transformation(&X);
        assert_complex_eq(qubit.get_zero_coef(), Complex64::new(0.0, 0.0));
        assert_complex_eq(qubit.get_one_coef(), Complex64::new(1.0, 0.0));
    }

    #[test]
    fn reset_returns_to_zero_state_after_transformations() {
        let mut qubit = Qubit::new();
        qubit.apply_transformation(&X);
        qubit.apply_transformation(&H);
        qubit.reset();
        assert_complex_eq(qubit.get_zero_coef(), Complex64::new(1.0, 0.0));
        assert_complex_eq(qubit.get_one_coef(), Complex64::new(0.0, 0.0));
    }

    #[test]
    fn hadamard_applied_twice_is_identity() {
        let mut qubit = Qubit::new();
        qubit.apply_transformation(&H);
        qubit.apply_transformation(&H);
        assert_complex_eq(qubit.get_zero_coef(), Complex64::new(1.0, 0.0));
        assert_complex_eq(qubit.get_one_coef(), Complex64::new(0.0, 0.0));
    }

    #[test]
    fn transformations_preserve_normalization() {
        let mut qubit = Qubit::new();
        qubit.apply_transformation(&H);
        qubit.apply_transformation(&X);
        qubit.apply_transformation(&H);
        let norm_sq = qubit.get_zero_coef().norm_sqr() + qubit.get_one_coef().norm_sqr();
        assert!((norm_sq - 1.0).abs() < EPS);
    }
}
