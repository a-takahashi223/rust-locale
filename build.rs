fn main() {
    cc::Build::new().file("c/src/ctype.c").compile("rustlocale");
    println!(r"cargo:rerun-if-changed=c/src/ctype.c");
}
