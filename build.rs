// https://doc.rust-lang.org/cargo/reference/build-scripts.html

// Standard
use std::env;
use std::path::Path;

// Constant
// https://doc.rust-lang.org/cargo/reference/environment-variables.html
const ENV_CARGO_CFG_TARGET_ARCH: &str = "CARGO_CFG_TARGET_ARCH";
const ENV_CARGO_CFG_TARGET_OS: &str = "CARGO_CFG_TARGET_OS";
const ENV_PROFILE: &str = "PROFILE";
const ENV_OUT_DIR: &str = "OUT_DIR";

macro_rules! cargo_warn{
    ($($tokens: tt)*) => {
        println!("cargo:warning=[{}] {}", file!(), format!($($tokens)*))
    }
}

fn main() {
    let profile = env::var(ENV_PROFILE).unwrap();
    let arch = env::var(ENV_CARGO_CFG_TARGET_ARCH).unwrap();
    let os = env::var(ENV_CARGO_CFG_TARGET_OS).unwrap();
    let current_dir = env::current_dir().unwrap();
    let out_dir = env::var(ENV_OUT_DIR).unwrap();
    let bin_location = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .display()
        .to_string();
    cargo_warn!("Current directory: {:}", current_dir.display());
    cargo_warn!("Profile: {:}", profile);
    cargo_warn!("Target architecture: {:}", arch);
    cargo_warn!("Target OS: {:}", os);
    cargo_warn!("Binary location: {:}", bin_location);
}
