use rust_kzg_bn254::blob::Blob;
use rust_kzg_bn254::kzg::Kzg;

// The commitment should match what is received from EigenDA's disperser as a commitment.
// The commitment received from dispersing kzgpad("hello") is
// x: LvAG1kdZAttu4Le86xzTDZGmZIgEuocTNYicLlTsLuA=
// y: Ez88I+rPb1gYjuepHJFaW9DtXIXzZKy0eEVFwKbwEtA=

pub fn main() {
    let mut kzg = Kzg::setup(true).unwrap();
    let mut blob = Blob::new(Vec::from("hello"));
    blob.pad_data().unwrap();
    let commitment = kzg.blob_to_kzg_commitment(&blob).unwrap();
    println!("Commitment: {:?}", commitment);
}
