use rust_kzg_bn254::kzg::Kzg;
use std::fs::File;
use std::io::BufWriter;
use ark_serialize::CanonicalSerialize;

pub fn main() {
    let kzg = Kzg::setup(true).unwrap();
    let file = File::create("src/test-files/g1_test.point").unwrap();
    let mut file = BufWriter::new(file);
    kzg.g1.serialize_uncompressed(&mut file).unwrap();
}
