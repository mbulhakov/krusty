#[cfg(unix)]
fn main() {
    println!("cargo:rustc-link-arg=-lgcc_s");
    println!("cargo:rustc-link-arg=-lutil");
    println!("cargo:rustc-link-arg=-lrt");
    println!("cargo:rustc-link-arg=-lpthread");
    println!("cargo:rustc-link-arg=-lm");
    println!("cargo:rustc-link-arg=-ldl");
    println!("cargo:rustc-link-arg=-lc");
}

#[cfg(windows)]
fn main() {}
