use rand::distributions::Alphanumeric;
use rand::Rng;

/// Generates a random alphanumeric string of the given length.
pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(|c| c as char)
        .collect()
}
