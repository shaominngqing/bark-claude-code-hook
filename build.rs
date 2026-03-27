fn main() {
    println!(
        "cargo:rustc-env=BARK_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );
    println!("cargo:rerun-if-changed=assets/bark-icon.png");
}
