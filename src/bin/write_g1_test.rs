use rust_kzg_bn254::kzg::Kzg;
use std::fs::File;
use std::io::{BufWriter, Write};

pub fn main() {
    let kzg = Kzg::setup(true).unwrap();
    let file = File::create("src/test-files/g1_test.point").unwrap();
    let mut file = BufWriter::new(file);
    for g1_point in kzg.g1.iter() {
        file.write_fmt(format_args!("{:?} ", g1_point.x.to_string())).unwrap();
        file.write_fmt(format_args!("{:?} ", g1_point.y.to_string())).unwrap();
    }
}
