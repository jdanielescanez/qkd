use crate::types::{ComplexMatrix, QuantumState};
use std::cell::RefCell;

thread_local! {
    static REGISTRY: RefCell<QuantumRegistry> = RefCell::new(QuantumRegistry { states: Vec::new() });
}

/// Owns every `QuantumState` created during the current quantum-communication round
/// on this thread. Reset at the start of each round (via `Qubit::reset_registry`) so
/// memory stays bounded by what a single round needs, regardless of how many rounds a
/// simulation runs overall.
struct QuantumRegistry {
    states: Vec<QuantumState>,
}

impl QuantumRegistry {
    /// Registers an already-built `QuantumState` and hands out one handle per each of
    /// its `qubit_count` qubits, all sharing the same `state_id`. The shared primitive
    /// behind every preset below (basis-state qubits, entangled pairs, and any future
    /// preset), so the "compute a state_id, push, build handles" bookkeeping lives in
    /// exactly one place.
    fn push_state(&mut self, state: QuantumState, qubit_count: usize) -> Vec<Qubit> {
        let state_id = self.states.len();
        self.states.push(state);
        (0..qubit_count).map(|index| Qubit { state_id, index }).collect()
    }

    fn push_basis_state_qubit(&mut self, value: bool) -> Qubit {
        self.push_state(QuantumState::create_basis_state(value), 1)[0]
    }

    fn push_entangled_state_qubits(&mut self) -> (Qubit, Qubit) {
        let qubits = self.push_state(QuantumState::create_bell_pair_state(), 2);
        (qubits[0], qubits[1])
    }
}

/// A reference to a single qubit within a `QuantumState` held in the thread-local
/// quantum registry: `state_id` identifies which `QuantumState` it belongs to, and
/// `index` identifies which of that state's qubits it is.
///
/// This is a plain pair of indices (`Copy`, no heap allocation, no shared pointer) —
/// the actual amplitudes always live in the registry, never in the `Qubit` itself. See
/// `QuantumState`'s doc comment for why that separation is what allows entanglement to
/// be represented at all.
#[derive(Copy, Clone)]
pub struct Qubit {
    state_id: usize,
    index: usize,
}

#[allow(dead_code)] // create_entangled_state is wired in once BBM92 is built (later phase)
impl Qubit {
    /// Creates a fresh single-qubit computational basis state (|0⟩/|1⟩) in the
    /// registry and returns a handle to it.
    pub fn create_basis_state(value: bool) -> Self {
        REGISTRY.with(|registry| registry.borrow_mut().push_basis_state_qubit(value))
    }

    /// Creates a fresh entangled state (a Bell pair) in the registry and returns a
    /// handle to each of its two qubits.
    pub(crate) fn create_entangled_state() -> (Self, Self) {
        REGISTRY.with(|registry| registry.borrow_mut().push_entangled_state_qubits())
    }

    /// Clears every `QuantumState` created so far on this thread. Must be called at
    /// the start of each quantum-communication round, before creating that round's
    /// qubits, so the registry never grows past what a single round needs.
    pub(crate) fn reset_registry() {
        REGISTRY.with(|registry| registry.borrow_mut().states.clear());
    }

    /// Applies a single-qubit gate to this qubit, within whatever (possibly
    /// multi-qubit, possibly entangled) state it belongs to.
    pub fn apply_local_gate(&self, gate: &ComplexMatrix) {
        REGISTRY.with(|registry| {
            registry.borrow_mut().states[self.state_id].apply_local_gate(self.index, gate)
        });
    }

    /// Measures this qubit in the computational basis, collapsing (and
    /// renormalizing) the whole state it belongs to, and returns the measured
    /// classical bit.
    pub fn measure(&self) -> bool {
        REGISTRY.with(|registry| registry.borrow_mut().states[self.state_id].measure(self.index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::H;
    use crate::rng::set_global_seed;

    #[test]
    fn create_basis_state_true_and_measure_round_trip() {
        set_global_seed(1);
        Qubit::reset_registry();
        let qubit = Qubit::create_basis_state(true);
        assert!(qubit.measure());
    }

    #[test]
    fn create_basis_state_false_and_measure_round_trip() {
        set_global_seed(2);
        Qubit::reset_registry();
        let qubit = Qubit::create_basis_state(false);
        assert!(!qubit.measure());
    }

    #[test]
    fn apply_local_gate_through_the_handle_matches_direct_state_behavior() {
        set_global_seed(3);
        Qubit::reset_registry();
        let qubit = Qubit::create_basis_state(false);
        qubit.apply_local_gate(&H);
        qubit.apply_local_gate(&H); // H is self-inverse
        assert!(!qubit.measure());
    }

    #[test]
    fn entangled_state_qubits_share_state_id_but_have_different_index() {
        Qubit::reset_registry();
        let (a, b) = Qubit::create_entangled_state();
        assert_eq!(a.state_id, b.state_id);
        assert_ne!(a.index, b.index);
    }

    #[test]
    fn entangled_state_measurements_are_perfectly_correlated_through_the_registry() {
        for seed in 0..50u64 {
            set_global_seed(seed);
            Qubit::reset_registry();
            let (a, b) = Qubit::create_entangled_state();
            assert_eq!(a.measure(), b.measure());
        }
    }

    #[test]
    fn reset_registry_frees_previous_states_so_memory_does_not_grow() {
        // Many rounds reusing the same registry: right after each reset, the number
        // of live states must be exactly what the current round created, never
        // accumulating leftovers from previous rounds.
        for _ in 0..1000 {
            Qubit::reset_registry();
            let _ = Qubit::create_basis_state(true);
            REGISTRY.with(|registry| {
                assert_eq!(registry.borrow().states.len(), 1);
            });
        }
    }

    #[test]
    fn state_ids_are_reused_after_a_reset() {
        Qubit::reset_registry();
        let first = Qubit::create_basis_state(true);
        Qubit::reset_registry();
        let second = Qubit::create_basis_state(true);
        assert_eq!(first.state_id, second.state_id);
    }
}
