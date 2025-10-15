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
        let divisor = divisor as f64;
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
