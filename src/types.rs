use crate::rng::rand_float;
use num_complex::Complex64;
use std::f64::consts::SQRT_2;
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

/// Represents the joint quantum state of `n` qubits as a vector of `2^n` complex
/// amplitudes over the computational basis, where bit `i` of a basis index gives the
/// state of qubit `i`.
///
/// Unlike a tuple of independent single-qubit states, a generic vector in this space
/// may be entangled: most vectors cannot be factored into a tensor product of
/// per-qubit states, which is exactly what makes this representation able to describe
/// entanglement. Qubits never own their amplitudes directly — see `registry::Qubit`,
/// which only ever holds a reference into a `QuantumState` owned by the registry.
pub(crate) struct QuantumState {
    amplitudes: Vec<Complex64>,
}

impl QuantumState {
    /// Creates a single-qubit computational basis state: |0⟩ for `value = false`,
    /// |1⟩ for `value = true`.
    pub(crate) fn create_basis_state(value: bool) -> Self {
        let (zero, one) = (Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0));
        let amplitudes = if value {
            vec![zero, one]
        } else {
            vec![one, zero]
        };
        QuantumState { amplitudes }
    }

    /// Creates the 2-qubit Bell state `(|00⟩ + |11⟩) / √2` directly, by injecting its
    /// amplitudes rather than building it up from gates. This state is entangled: it
    /// cannot be factored into a product of two single-qubit states.
    pub(crate) fn create_bell_pair_state() -> Self {
        let amplitude = Complex64::new(1.0 / SQRT_2, 0.0);
        let zero = Complex64::new(0.0, 0.0);
        QuantumState {
            amplitudes: vec![amplitude, zero, zero, amplitude],
        }
    }

    /// Applies a single-qubit gate to qubit `qubit_index` within this state, leaving
    /// every other qubit untouched. Groups the amplitudes into pairs that differ only
    /// in bit `qubit_index` and applies `gate` to each pair, which is equivalent to
    /// (but far cheaper than) tensoring `gate` up to the full `2^n`-dimensional space.
    pub(crate) fn apply_local_gate(&mut self, qubit_index: usize, gate: &ComplexMatrix) {
        let mask = 1usize << qubit_index;
        for base in 0..self.amplitudes.len() {
            if base & mask != 0 {
                continue; // only process each {i0, i1} pair once, from its i0 side
            }
            let (i0, i1) = (base, base | mask);
            let (a0, a1) = (self.amplitudes[i0], self.amplitudes[i1]);
            self.amplitudes[i0] = a0 * gate.0[0][0] + a1 * gate.0[0][1];
            self.amplitudes[i1] = a0 * gate.0[1][0] + a1 * gate.0[1][1];
        }
    }

    /// Measures qubit `qubit_index` in the computational basis: samples the outcome
    /// according to the summed probability over every basis state consistent with
    /// each result, then collapses and renormalizes this state to match, and returns
    /// the measured classical bit. For an entangled state, this also affects the
    /// amplitudes of every other qubit sharing it.
    pub(crate) fn measure(&mut self, qubit_index: usize) -> bool {
        let mask = 1usize << qubit_index;
        let one_probability: f64 = self
            .amplitudes
            .iter()
            .enumerate()
            .filter(|(i, _)| i & mask != 0)
            .map(|(_, amplitude)| amplitude.norm_sqr())
            .sum();

        let outcome = rand_float() < one_probability;
        let surviving_bit = if outcome { mask } else { 0 };

        let mut norm_sq = 0.0;
        for (i, amplitude) in self.amplitudes.iter_mut().enumerate() {
            if i & mask != surviving_bit {
                *amplitude = Complex64::new(0.0, 0.0);
            } else {
                norm_sq += amplitude.norm_sqr();
            }
        }
        let norm = norm_sq.sqrt();
        for amplitude in self.amplitudes.iter_mut() {
            *amplitude /= norm;
        }

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{H, I, X};
    use crate::rng::set_global_seed;

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
    fn apply_local_gate_hadamard_twice_is_identity() {
        let mut state = QuantumState::create_basis_state(false);
        state.apply_local_gate(0, &H);
        state.apply_local_gate(0, &H);
        assert_complex_eq(state.amplitudes[0], Complex64::new(1.0, 0.0));
        assert_complex_eq(state.amplitudes[1], Complex64::new(0.0, 0.0));
    }

    #[test]
    fn apply_local_gate_preserves_normalization() {
        let mut state = QuantumState::create_basis_state(false);
        state.apply_local_gate(0, &H);
        state.apply_local_gate(0, &X);
        state.apply_local_gate(0, &H);
        let norm_sq: f64 = state.amplitudes.iter().map(|a| a.norm_sqr()).sum();
        assert!((norm_sq - 1.0).abs() < EPS);
    }

    // Reinterprets a 2-qubit state's 4 amplitudes as a 2x2 matrix: the state is
    // separable (not entangled) iff that matrix has rank 1 (determinant == 0).
    fn is_separable_pair(state: &QuantumState) -> bool {
        let a = &state.amplitudes;
        let det = a[0] * a[3] - a[1] * a[2];
        det.norm() < EPS
    }

    #[test]
    fn create_basis_state_matches_zero_and_one() {
        let zero = QuantumState::create_basis_state(false);
        assert_complex_eq(zero.amplitudes[0], Complex64::new(1.0, 0.0));
        assert_complex_eq(zero.amplitudes[1], Complex64::new(0.0, 0.0));

        let one = QuantumState::create_basis_state(true);
        assert_complex_eq(one.amplitudes[0], Complex64::new(0.0, 0.0));
        assert_complex_eq(one.amplitudes[1], Complex64::new(1.0, 0.0));
    }

    #[test]
    fn apply_local_gate_on_single_qubit_matches_direct_transformation() {
        // Parity check against Qubit::apply_transformation (today's n=1 behavior).
        let mut state = QuantumState::create_basis_state(false);
        state.apply_local_gate(0, &H);
        assert_complex_eq(state.amplitudes[0], Complex64::new(1.0 / SQRT_2, 0.0));
        assert_complex_eq(state.amplitudes[1], Complex64::new(1.0 / SQRT_2, 0.0));
    }

    #[test]
    fn apply_local_gate_touches_only_the_targeted_qubit() {
        // Product state with qubit0 = 1, qubit1 = 0 (index 1 = binary 01, bit0 =
        // qubit0). Applying H to qubit 1 must leave qubit 0's definite value alone.
        let mut state = QuantumState {
            amplitudes: vec![
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
        };
        state.apply_local_gate(1, &H);
        // Expected: (|01> + |11>) / sqrt(2) -> indices 1 and 3, qubit0 still always 1.
        assert_complex_eq(state.amplitudes[0], Complex64::new(0.0, 0.0));
        assert_complex_eq(state.amplitudes[1], Complex64::new(1.0 / SQRT_2, 0.0));
        assert_complex_eq(state.amplitudes[2], Complex64::new(0.0, 0.0));
        assert_complex_eq(state.amplitudes[3], Complex64::new(1.0 / SQRT_2, 0.0));
    }

    #[test]
    fn measure_zero_state_always_returns_false() {
        set_global_seed(1);
        for _ in 0..50 {
            let mut state = QuantumState::create_basis_state(false);
            assert!(!state.measure(0));
        }
    }

    #[test]
    fn measure_one_state_always_returns_true() {
        set_global_seed(2);
        for _ in 0..50 {
            let mut state = QuantumState::create_basis_state(true);
            assert!(state.measure(0));
        }
    }

    #[test]
    fn measure_collapses_and_renormalizes_to_the_observed_outcome() {
        set_global_seed(3);
        for seed in 0..20u64 {
            set_global_seed(seed);
            let mut state = QuantumState::create_basis_state(false);
            state.apply_local_gate(0, &H);
            let outcome = state.measure(0);
            let (expected_zero, expected_one) = if outcome {
                (Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0))
            } else {
                (Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0))
            };
            assert_complex_eq(state.amplitudes[0], expected_zero);
            assert_complex_eq(state.amplitudes[1], expected_one);
        }
    }

    #[test]
    fn create_bell_pair_state_is_entangled() {
        let state = QuantumState::create_bell_pair_state();
        assert!(!is_separable_pair(&state));
    }

    #[test]
    fn create_bell_pair_state_is_normalized_and_has_equal_weight_on_00_and_11() {
        let state = QuantumState::create_bell_pair_state();
        let norm_sq: f64 = state.amplitudes.iter().map(|a| a.norm_sqr()).sum();
        assert!((norm_sq - 1.0).abs() < EPS);
        assert_complex_eq(state.amplitudes[0], state.amplitudes[3]);
        assert_complex_eq(state.amplitudes[1], Complex64::new(0.0, 0.0));
        assert_complex_eq(state.amplitudes[2], Complex64::new(0.0, 0.0));
    }

    #[test]
    fn a_product_state_is_detected_as_separable() {
        // Sanity check on the separability helper itself, using (|0>+|1>)/sqrt(2) ⊗ |0>.
        let state = QuantumState {
            amplitudes: vec![
                Complex64::new(1.0 / SQRT_2, 0.0),
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0 / SQRT_2, 0.0),
                Complex64::new(0.0, 0.0),
            ],
        };
        assert!(is_separable_pair(&state));
    }

    #[test]
    fn measuring_a_bell_pair_gives_perfectly_correlated_outcomes() {
        for seed in 0..50u64 {
            set_global_seed(seed);
            let mut state = QuantumState::create_bell_pair_state();
            let first = state.measure(0);
            let second = state.measure(1);
            assert_eq!(first, second);
        }
    }
}
