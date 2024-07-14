fn main() {
    println!("cargo::rerun-if-changed=/home/hawkinsw/cerfcast-ready4l4s/bpf/build/block.o");
    println!("cargo::rerun-if-changed=bpf/link");
    println!("cargo::rustc-link-arg=-Wl,-Tbpf/link");
}