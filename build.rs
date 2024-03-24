#[cfg(windows)]
fn main() {
    println!("cargo:rustc-link-lib=./resources/res");
}

#[cfg(not(windows))]
fn main() {}
