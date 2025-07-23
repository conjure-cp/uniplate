pub fn main() {
    println!("cargo::rustc-check-cfg=cfg(uniplate_trace, values(\"walkinto\"))");
}
