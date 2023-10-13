// https://doc.rust-lang.org/cargo/reference/build-scripts.html

// Standard
use std::env;

// Constant
const ENV_PROFILE: &str = "PROFILE";

macro_rules! cargo_warn{
    ($($tokens: tt)*) => {
        println!("cargo:warning=[{}] {}", file!(), format!($($tokens)*))
    }
}

fn main() {
    let profile = env::var(ENV_PROFILE).unwrap();
    cargo_warn!("{:}: {:}", ENV_PROFILE, profile);
}
