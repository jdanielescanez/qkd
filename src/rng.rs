use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::cell::RefCell;

// Credits to qcrypto crate (J. Garcia-Diaz and D. Escanez-Exposito)

thread_local! {
    static GLOBAL_RNG: RefCell<ChaCha8Rng> = RefCell::new(ChaCha8Rng::from_os_rng());
}

/// Sets a deterministic seed for all random operations on the current thread.
pub fn set_global_seed(seed: u64) {
    GLOBAL_RNG.with(|rng| {
        *rng.borrow_mut() = ChaCha8Rng::seed_from_u64(seed);
    });
}

pub(crate) fn rand_float() -> f64 {
    GLOBAL_RNG.with(|rng| rng.borrow_mut().random())
}

pub(crate) fn rand_bool() -> bool {
    GLOBAL_RNG.with(|rng| rng.borrow_mut().random_bool(0.5))
}

pub(crate) fn rand_choose<T: Clone>(vec: Vec<T>) -> T {
    GLOBAL_RNG.with(|rng| {
        vec.choose(&mut *rng.borrow_mut())
            .cloned()
            .expect("Vec cannot be empty")
    })
}

pub(crate) fn shuffle_and_split<T: Clone>(mut vector: Vec<T>) -> (Vec<T>, Vec<T>) {
    GLOBAL_RNG.with(|rng| vector.shuffle(&mut *rng.borrow_mut()));
    let half = vector.len() / 2;
    let first_half = vector[..half].to_vec();
    let second_half = vector[half..].to_vec();
    (first_half, second_half)
}

#[cfg(test)]
mod tests {
    use super::*;

    // GLOBAL_RNG is thread_local, and each #[test] fn runs on its own thread, so
    // set_global_seed here does not affect other tests running concurrently.

    #[test]
    fn same_seed_reproduces_the_same_sequence() {
        set_global_seed(42);
        let seq1: Vec<f64> = (0..10).map(|_| rand_float()).collect();
        set_global_seed(42);
        let seq2: Vec<f64> = (0..10).map(|_| rand_float()).collect();
        assert_eq!(seq1, seq2);
    }

    #[test]
    fn different_seeds_diverge() {
        set_global_seed(1);
        let seq1: Vec<f64> = (0..10).map(|_| rand_float()).collect();
        set_global_seed(2);
        let seq2: Vec<f64> = (0..10).map(|_| rand_float()).collect();
        assert_ne!(seq1, seq2);
    }

    #[test]
    fn rand_float_stays_within_unit_range() {
        set_global_seed(3);
        for _ in 0..1000 {
            let value = rand_float();
            assert!((0.0..1.0).contains(&value));
        }
    }

    #[test]
    fn rand_choose_with_single_element_always_returns_it() {
        set_global_seed(4);
        for _ in 0..20 {
            assert_eq!(rand_choose(vec![42]), 42);
        }
    }

    #[test]
    #[should_panic(expected = "Vec cannot be empty")]
    fn rand_choose_with_empty_vec_panics() {
        let _: i32 = rand_choose(vec![]);
    }

    #[test]
    fn shuffle_and_split_preserves_all_elements() {
        set_global_seed(5);
        let input: Vec<i32> = (0..11).collect();
        let (first, second) = shuffle_and_split(input.clone());
        let mut combined = first;
        combined.extend(second);
        combined.sort();
        assert_eq!(combined, input);
    }

    #[test]
    fn shuffle_and_split_even_length_splits_evenly() {
        set_global_seed(6);
        let input: Vec<i32> = (0..10).collect();
        let (first, second) = shuffle_and_split(input);
        assert_eq!(first.len(), 5);
        assert_eq!(second.len(), 5);
    }

    #[test]
    fn shuffle_and_split_odd_length_gives_first_half_the_floor() {
        set_global_seed(7);
        let input: Vec<i32> = (0..7).collect();
        let (first, second) = shuffle_and_split(input);
        assert_eq!(first.len(), 3);
        assert_eq!(second.len(), 4);
    }

    #[test]
    fn shuffle_and_split_empty_vec_gives_two_empty_vecs() {
        let (first, second) = shuffle_and_split(Vec::<i32>::new());
        assert!(first.is_empty());
        assert!(second.is_empty());
    }
}
