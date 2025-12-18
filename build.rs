fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS") == Ok("android".into()) {
        println!("cargo:rustc-link-lib=dylib=c++");
    }
    
}