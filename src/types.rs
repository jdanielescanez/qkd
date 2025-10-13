use num_complex::Complex64;
use std::ops::{Add, Div};

// TODO: Use standard library for matrices
#[derive(Clone, Copy)]
pub struct ComplexMatrix(pub [[Complex64; 2]; 2]);

impl ComplexMatrix {
    pub fn invert(&self) -> Option<ComplexMatrix> {
        let a = self.0[0][0];
        let b = self.0[0][1];
        let c = self.0[1][0];
        let d = self.0[1][1];

        let det = a * d - b * c;
        if det == Complex64::new(0.0, 0.0) {
            return None; // Not invertible
        }

        let inv_det = Complex64::new(1.0, 0.0) / det;
        Some(ComplexMatrix([
            [d * inv_det, -b * inv_det],
            [-c * inv_det, a * inv_det],
        ]))
    }
}

impl Add<ComplexMatrix> for ComplexMatrix {
    type Output = Self;

    fn add(self, matrix: Self) -> Self::Output {
        ComplexMatrix([
            [self.0[0][0] + matrix.0[0][0], self.0[0][1] + matrix.0[0][1]],
            [self.0[1][0] + matrix.0[1][0], self.0[1][1] + matrix.0[1][1]],
        ])
    }
}

impl Into<ComplexMatrix> for [[Complex64; 2]; 2] {
    fn into(self) -> ComplexMatrix {
        ComplexMatrix(self)
    }
}

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

pub struct Qubit {
    state: (Complex64, Complex64),
}

impl Qubit {
    pub fn new() -> Self {
        Qubit {
            state: (Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)),
        }
    }

    pub fn reset(&mut self) {
        *self = Qubit::new();
    }

    pub fn apply_transformation(&mut self, matrix: &ComplexMatrix) {
        self.state = (
            self.state.0 * matrix.0[0][0] + self.state.1 * matrix.0[0][1],
            self.state.0 * matrix.0[1][0] + self.state.1 * matrix.0[1][1],
        );
    }

    pub fn get_zero_coef(&self) -> Complex64 {
        self.state.0
    }

    pub fn get_one_coef(&self) -> Complex64 {
        self.state.1
    }
}
