// https://doc.rust-lang.org/cargo/reference/build-scripts.html

// Standard
use std::env;

// Constant
// https://doc.rust-lang.org/cargo/reference/environment-variables.html
const ENV_CARGO_CFG_TARGET_ARCH: &str = "CARGO_CFG_TARGET_ARCH";
const ENV_CARGO_CFG_TARGET_OS: &str = "CARGO_CFG_TARGET_OS";
const ENV_PROFILE: &str = "PROFILE";

macro_rules! cargo_warn{
    ($($tokens: tt)*) => {
        println!("cargo:warning=[{}] {}", file!(), format!($($tokens)*))
    }
}

fn main() {
    let profile = env::var(ENV_PROFILE).unwrap();
    let arch = env::var(ENV_CARGO_CFG_TARGET_ARCH).unwrap();
    let os = env::var(ENV_CARGO_CFG_TARGET_OS).unwrap();
    cargo_warn!("Profile: {:}", profile);
    cargo_warn!("Target Arch: {:}", arch);
    cargo_warn!("Target OS: {:}", os);
}
