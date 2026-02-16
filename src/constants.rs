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
