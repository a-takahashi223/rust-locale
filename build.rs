use std::process::Command;

fn main() {
    let _ = Command::new("make").args(["-C", "c", "clean"]).status();
    let _ = Command::new("make").args(["-C", "c", "distclean"]).status();
    Command::new(std::fs::canonicalize("c/configure").unwrap())
        .current_dir("c")
        .status()
        .unwrap();
    Command::new("make").args(["-C", "c"]).status().unwrap();
    println!(r"cargo:rustc-link-search=c/src");
    println!(r"cargo:rustc-link-search=c/lib");
    println!(r"cargo:rustc-link-lib=static=gnu");
    println!(r"cargo:rerun-if-changed=c/src/ctype.c");
}
