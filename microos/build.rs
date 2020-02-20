fn main() {
    println!("cargo:rustc-cfg=riscv");
    println!("cargo:rustc-cfg=riscv32");
    // println!(r#"cargo:rustc-cfg=boot="pxeboot""#); // these two set for rust compile of source #[cfg(...)]
    println!(r#"cargo:rustc-cfg=boot="root""#);
}
