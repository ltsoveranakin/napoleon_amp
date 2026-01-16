fn main() {
    println!("cargo:rustc-link-arg=-landroid");
    println!("cargo:rustc-link-arg=-laudio");
    println!("cargo:rustc-link-arg=-llog");
    println!("cargo:rustc-link-arg=-ldl");
}
