use std::env;
use std::fs::File;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use std::process::Command;

fn main() {
    // Put the linker script somewhere the linker can find it
    let out_dir = env::var("OUT_DIR").unwrap();
    let out     = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target  = env::var("TARGET").unwrap();
    let name    = "riscv_base";

    println!("cargo:rerun-if-changed=src/init.S");
    println!("cargo:rerun-if-changed=src/utils.S");
    Command::new("riscv32-elf-gcc").args(&["-c",
                                           "-mabi=ilp32",
                                           "-march=rv32imac",
                                           "src/init.S",
                                           "-o"])
        .arg(&format!("{}/init.o", out_dir))
        .status().unwrap();
    Command::new("riscv32-elf-gcc").args(&["-c",
                                           "-mabi=ilp32",
                                           "-march=rv32imac",
                                           "src/utils.S",
                                           "-o"])
        .arg(&format!("{}/utils.o", out_dir))
        .status().unwrap();
    Command::new("riscv32-elf-ar").args(&["crs"])
        .arg(&format!("lib{}.a", name))
        .args(&["init.o"])
        .args(&["utils.o"])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", name);

    File::create(out.join("link.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=link.x");
    println!("cargo:rustc-cfg=riscv");
    println!("cargo:rustc-cfg=riscv32");
}
