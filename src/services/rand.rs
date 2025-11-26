use rand::{Rng, distr::Alphanumeric};

pub fn generate_random_id() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
