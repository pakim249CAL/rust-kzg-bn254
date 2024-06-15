use ark_serialize::CanonicalDeserialize;
use ark_bn254::g1::G1Affine;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::writeln;

const G1_MAINNET_BYTES: &[u8; 8_388_616] = include_bytes!("../test-files/g1_mainnet.point");
const G1_TEST_BYTES: &[u8; 192_008] = include_bytes!("../test-files/g1_test.point");

fn main() {
    let g1: Vec<G1Affine> = CanonicalDeserialize::deserialize_uncompressed_unchecked(G1_TEST_BYTES.as_slice()).unwrap();
    let file = File::create("../generated/g1_const.rs").unwrap();
    let mut file = BufWriter::new(file);

    writeln!(
        file,
        "use ark_bn254::g1::G1Affine;\nuse ark_ff::MontFp;\n"
    ).unwrap();

    writeln!(file, "pub const G1: &[G1Affine] = &[").unwrap();

    for g1_point in g1.iter() {
        writeln!(
            &mut file,
            "G1Affine::new_unchecked(MontFp!({:?}), MontFp!({:?})),",
            g1_point.x.to_string(),
            g1_point.y.to_string(),
        ).unwrap();
    }

    writeln!(&mut file, "];").unwrap();
    for i in 0..g1.len() {
        println!("{:?}", g1[i]);
    }
}