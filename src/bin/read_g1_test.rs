use rust_kzg_bn254::kzg::Kzg;
use ark_bn254::g1::G1Affine;
use ark_bn254::{Bn254, Fr, G1Projective, G2Affine};
use ark_ff::{const_for, MontFp, Fp};
use std::str::FromStr;

const G1_POINTS_LENGTH: usize = 3000;

const G1_POINTS_STR: &str = include_str!("../test-files/g1_test.point");

const fn G1_POINTS() -> [G1Affine; G1_POINTS_LENGTH] {
    let mut start = 0;
    let mut end = 0;

    // first entry is length
    (start, end) = get_next_entry(G1_POINTS_STR, 0);

    let mut g1_points = [G1Affine::new_unchecked(MontFp!("0"), MontFp!("0")); G1_POINTS_LENGTH];

    const_for!((i in 0..(G1_POINTS_LENGTH)) {
        (start, end) = get_next_entry(G1_POINTS_STR, end);
        let x = MontFp!(&G1_POINTS_STR[start..end]);
        (start, end) = get_next_entry(G1_POINTS_STR, end);
        let y = Fp::from_str(&G1_POINTS_STR[start..end]).unwrap();
        g1_points[i] = G1Affine::new_unchecked(x, y);
    });
    g1_points
}

const fn get_next_entry(s: &str, start: usize) -> (usize, usize) {
    let mut end = start;
    while s.as_bytes()[end] != b' ' {
        end += 1;
    }
    (start, end)
}

pub fn main() {
    let kzg = Kzg::setup(true).unwrap();
}
