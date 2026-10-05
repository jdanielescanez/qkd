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

/// Parameterized single-qubit measurement-basis rotation.
///
/// Returns the real, symmetric, unitary matrix:
/// ```text
/// |  cos θ   sin θ |
/// |  sin θ  -cos θ |
/// ```
/// This is a reflection (determinant -1), not a rotation in the `SO(2)` sense — it is the
/// member of a one-parameter family of involutions (`rotation(θ) · rotation(θ) == I` for
/// every `θ`) that maps the eigenbasis of "measure along the axis at angle `θ`" onto the
/// computational basis, the same role `H` plays for the X-basis.
///
/// `I` is *not* a member of this family (its determinant is +1, not -1), so `rotation`
/// does not reduce to `I` at `theta = 0.0` — it reduces to the Pauli-Z matrix there
/// instead, which is measurement-equivalent to `I` for a single qubit (it only flips the
/// sign of the |1⟩ amplitude, leaving `|amplitude|²` — and so every measurement
/// probability — unchanged). Two angles do line up with the crate's existing fixed-angle
/// constants exactly: `rotation(PI / 4.0) == H` and `rotation(PI / 2.0) == X`.
pub fn rotation(theta: f64) -> ComplexMatrix {
    let (cos, sin) = (theta.cos(), theta.sin());
    ComplexMatrix([
        [Complex64::new(cos, 0.0), Complex64::new(sin, 0.0)],
        [Complex64::new(sin, 0.0), Complex64::new(-cos, 0.0)],
    ])
}

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

    fn assert_complex_eq(actual: Complex64, expected: Complex64) {
        assert!(
            (actual - expected).norm() < EPS,
            "expected {expected:?}, got {actual:?}"
        );
    }

    #[test]
    fn rotation_is_unitary_and_self_inverse_for_many_angles() {
        for i in 0..=16 {
            let theta = std::f64::consts::PI * (i as f64) / 8.0; // 0 .. 2*PI in steps of PI/8
            let r = rotation(theta);
            assert_is_identity(matmul(&r, &conjugate_transpose(&r)));
            assert_is_identity(matmul(&r, &r));
        }
    }

    #[test]
    fn rotation_at_pi_over_4_equals_h() {
        let r = rotation(std::f64::consts::PI / 4.0);
        for row in 0..2 {
            for col in 0..2 {
                assert_complex_eq(r.0[row][col], H.0[row][col]);
            }
        }
    }

    #[test]
    fn rotation_at_pi_over_2_equals_x() {
        let r = rotation(std::f64::consts::PI / 2.0);
        for row in 0..2 {
            for col in 0..2 {
                assert_complex_eq(r.0[row][col], X.0[row][col]);
            }
        }
    }

    #[test]
    fn rotation_at_zero_is_measurement_equivalent_to_identity() {
        // rotation(0.0) is the Pauli-Z matrix, not I: it only flips the sign of the |1>
        // amplitude, which leaves every |amplitude|^2 (and so every measurement
        // probability) unchanged.
        let r = rotation(0.0);
        assert_complex_eq(r.0[0][0], Complex64::new(1.0, 0.0));
        assert_complex_eq(r.0[1][1], Complex64::new(-1.0, 0.0));
        assert_complex_eq(r.0[0][1], Complex64::new(0.0, 0.0));
        assert_complex_eq(r.0[1][0], Complex64::new(0.0, 0.0));
    }
}
