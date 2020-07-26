use prost_build::compile_protos;

fn main() {
    compile_protos(&["proto/aqmm.proto"], &["proto/"]).unwrap();
}
