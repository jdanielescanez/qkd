use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use rand::{rng, Rng};

/// Randomly selects an element from a vector.
///
/// # Arguments
///
/// * `vec` - A non-empty vector of elements to choose from.
///
/// # Returns
///
/// A randomly selected element from the vector.
///
/// # Panics
///
/// Panics if the input vector is empty.
pub fn rand_choose<T: Clone>(vec: Vec<T>) -> T {
    let mut rng = rng();
    vec.choose(&mut rng).cloned().expect("Vec cannot be empty")
}

/// Generates a random boolean value.
///
/// # Returns
///
/// `true` or `false` with equal probability (50% each).
pub fn rand_bool() -> bool {
    let mut rng = rng();
    *[true, false].choose(&mut rng).unwrap()
}

/// Generates a random floating-point number in the range [0, 1).
///
/// # Returns
///
/// A random `f64` value uniformly distributed in the interval [0, 1).
pub fn rand_float() -> f64 {
    rng().random()
}

/// Randomly shuffles a vector and splits it into two halves.
///
/// # Arguments
///
/// * `vector` - The vector to shuffle and split.
///
/// # Returns
///
/// A tuple containing two new vectors:
/// - The first half of the shuffled vector.
/// - The second half of the shuffled vector.
pub fn shuffle_and_split<T>(mut vector: Vec<T>) -> (Vec<T>, Vec<T>)
where
    T: Clone,
{
    let mut rng = rng();
    vector.shuffle(&mut rng);
    let half = vector.len() / 2;
    let first_half = vector[..half].to_vec();
    let second_half = vector[half..].to_vec();
    (first_half, second_half)
}
