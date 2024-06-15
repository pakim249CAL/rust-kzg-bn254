use rust_kzg_bn254::kzg::Kzg;
use std::fs::File;
use std::io::{BufWriter, Write};

pub fn main() {
    let kzg = Kzg::setup(false).unwrap();
    println!("{:?}", kzg.g1.len());
}
