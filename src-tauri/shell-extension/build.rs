fn main() {
    // No special build steps needed for shell extension
    println!("cargo:rerun-if-changed=src/lib.rs");
}