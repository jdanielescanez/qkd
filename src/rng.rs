use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::cell::RefCell;

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
    GLOBAL_RNG.with(|rng| vec.choose(&mut *rng.borrow_mut()).cloned().expect("Vec cannot be empty"))
}

pub(crate) fn shuffle_and_split<T: Clone>(mut vector: Vec<T>) -> (Vec<T>, Vec<T>) {
    GLOBAL_RNG.with(|rng| vector.shuffle(&mut *rng.borrow_mut()));
    let half = vector.len() / 2;
    let first_half = vector[..half].to_vec();
    let second_half = vector[half..].to_vec();
    (first_half, second_half)
}
