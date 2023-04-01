fn main() {
    if std::env::var("CI").is_ok() {
        println!("cargo:rustc-cfg=feature=\"test_ci\"");
    }
    // std::process::exit(1);
}
