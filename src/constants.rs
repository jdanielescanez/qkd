use crate::types::ComplexMatrix;
use num_complex::Complex64;
use std::f64::consts::SQRT_2;

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

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    fn conjugate_transpose(m: &ComplexMatrix) -> ComplexMatrix {
        ComplexMatrix([
            [m.0[0][0].conj(), m.0[1][0].conj()],
            [m.0[0][1].conj(), m.0[1][1].conj()],
        ])
    }

    fn matmul(a: &ComplexMatrix, b: &ComplexMatrix) -> ComplexMatrix {
        let mut out = [[Complex64::new(0.0, 0.0); 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                out[i][j] = a.0[i][0] * b.0[0][j] + a.0[i][1] * b.0[1][j];
            }
        }
        ComplexMatrix(out)
    }

    fn assert_is_identity(m: ComplexMatrix) {
        assert!((m.0[0][0] - Complex64::new(1.0, 0.0)).norm() < EPS);
        assert!((m.0[1][1] - Complex64::new(1.0, 0.0)).norm() < EPS);
        assert!(m.0[0][1].norm() < EPS);
        assert!(m.0[1][0].norm() < EPS);
    }

    #[test]
    fn i_is_unitary() {
        assert_is_identity(matmul(&I, &conjugate_transpose(&I)));
    }

    #[test]
    fn h_is_unitary_and_self_inverse() {
        assert_is_identity(matmul(&H, &conjugate_transpose(&H)));
        assert_is_identity(matmul(&H, &H));
    }

    #[test]
    fn x_is_unitary_and_self_inverse() {
        assert_is_identity(matmul(&X, &conjugate_transpose(&X)));
        assert_is_identity(matmul(&X, &X));
    }

    #[test]
    fn h_y_is_unitary() {
        assert_is_identity(matmul(&H_Y, &conjugate_transpose(&H_Y)));
    }

    #[test]
    fn h_y_is_invertible_and_its_inverse_undoes_it() {
        // build_six_state() relies on H_Y.invert().unwrap() never panicking.
        let inv = H_Y.invert().expect("H_Y must be invertible");
        assert_is_identity(matmul(&H_Y, &inv));
        assert_is_identity(matmul(&inv, &H_Y));
    }
}
