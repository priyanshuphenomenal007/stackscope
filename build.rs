// build.rs
fn main() {
    // Tells Cargo to rerun this build script only if build.rs changes.
    // In future phases, we will add tracking for test fixture C/Rust files here.
    println!("cargo:rerun-if-changed=build.rs");
}
