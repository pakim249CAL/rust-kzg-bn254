use crate::blob::Blob;
use crate::consts::BYTES_PER_FIELD_ELEMENT;
use crate::errors::KzgError;
use crate::polynomial::Polynomial;
use ark_bn254::g1::G1Affine;
use ark_bn254::{Bn254, Fr, G1Projective, G2Affine};
use ark_ec::pairing::Pairing;
use ark_ec::{AffineRepr, CurveGroup, VariableBaseMSM};
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::ops::{Div, Mul};
use ark_std::str::FromStr;
use ark_std::{One, Zero};
use num_traits::ToPrimitive;

#[derive(Debug, PartialEq, Clone, CanonicalSerialize, CanonicalDeserialize)]
pub struct Kzg {
    pub g1: Vec<G1Affine>,
    pub g2: Vec<G2Affine>,
    pub params: Params,
    pub srs_order: u64,
    pub expanded_roots_of_unity: Vec<Fr>,
}

#[derive(Debug, PartialEq, Clone, CanonicalSerialize, CanonicalDeserialize)]
pub struct Params {
    chunk_length: u64,
    num_chunks: u64,
    max_fft_width: u64,
    completed_setup: bool,
}

const KZG_MAINNET_BYTES: &[u8; 4_196_153] = include_bytes!("test-files/kzg_serialized_mainnet");

const KZG_TEST_BYTES: &[u8; 288_057] = include_bytes!("test-files/kzg_serialized_test");

impl Kzg {
    pub fn setup(test: bool) -> Result<Self, KzgError> {
        if test {
            let kzg: Kzg =
                CanonicalDeserialize::deserialize_compressed(KZG_TEST_BYTES.as_slice()).unwrap();
            Ok(Self {
                g1: kzg.g1,
                g2: kzg.g2,
                params: kzg.params,
                srs_order: kzg.srs_order,
                expanded_roots_of_unity: kzg.expanded_roots_of_unity,
            })
        } else {
            let kzg: Kzg =
                CanonicalDeserialize::deserialize_compressed(KZG_MAINNET_BYTES.as_slice()).unwrap();
            Ok(Self {
                g1: kzg.g1,
                g2: kzg.g2,
                params: kzg.params,
                srs_order: kzg.srs_order,
                expanded_roots_of_unity: kzg.expanded_roots_of_unity,
            })
        }
    }

    /// data_setup_custom is a helper function
    pub fn data_setup_custom(
        &mut self,
        num_of_nodes: u64,
        padded_input_data_size: u64,
    ) -> Result<(), KzgError> {
        let floor = u64::try_from(BYTES_PER_FIELD_ELEMENT)
            .map_err(|e| KzgError::SerializationError(e.to_string()))?;
        let len_of_data_in_elements = padded_input_data_size.div_ceil(floor);
        let min_num_chunks = len_of_data_in_elements.div_ceil(num_of_nodes);
        self.data_setup_mins(min_num_chunks, num_of_nodes)
    }

    ///data_setup_mins sets up the environment per the blob data
    pub fn data_setup_mins(
        &mut self,
        min_chunk_length: u64,
        min_num_chunks: u64,
    ) -> Result<(), KzgError> {
        let mut params = Params {
            num_chunks: min_num_chunks.next_power_of_two(),
            chunk_length: min_chunk_length.next_power_of_two(),
            max_fft_width: 0_u64,
            completed_setup: false,
        };

        let number_of_evaluations = params.chunk_length * params.num_chunks;
        let mut log2_of_evals = number_of_evaluations
            .to_f64()
            .unwrap()
            .log2()
            .to_u8()
            .unwrap();
        params.max_fft_width = 1_u64 << log2_of_evals;

        if params.chunk_length == 1 {
            log2_of_evals = (2 * params.num_chunks)
                .to_f64()
                .unwrap()
                .log2()
                .to_u8()
                .unwrap();
        }

        if params.chunk_length * params.num_chunks >= self.srs_order {
            return Err(KzgError::SerializationError(
                "the supplied encoding parameters are not valid with respect to the SRS."
                    .to_string(),
            ));
        }

        let primitive_roots_of_unity = Self::get_primitive_roots_of_unity();
        let found_root_of_unity = primitive_roots_of_unity
            .get(log2_of_evals.to_usize().unwrap())
            .unwrap();
        let mut expanded_roots_of_unity = Self::expand_root_of_unity(found_root_of_unity);
        expanded_roots_of_unity.truncate(expanded_roots_of_unity.len() - 1);

        params.completed_setup = true;
        self.params = params;
        self.expanded_roots_of_unity = expanded_roots_of_unity;

        Ok(())
    }

    /// helper function to get the
    pub fn get_nth_root_of_unity(&self, i: usize) -> Option<&Fr> {
        self.expanded_roots_of_unity.get(i)
    }

    ///function to expand the roots based on the configuration
    fn expand_root_of_unity(root_of_unity: &Fr) -> Vec<Fr> {
        let mut roots = vec![Fr::one()]; // Initialize with 1
        roots.push(*root_of_unity); // Add the root of unity

        let mut i = 1;
        while !roots[i].is_one() {
            // Continue until the element cycles back to one
            let this = &roots[i];
            i += 1;
            roots.push(this * root_of_unity); // Push the next power of the root of unity
        }
        roots
    }

    /// refer to DA code for more context
    fn get_primitive_roots_of_unity() -> Vec<Fr> {
        let data: [&str; 29] = [
            "1",
            "21888242871839275222246405745257275088548364400416034343698204186575808495616",
            "21888242871839275217838484774961031246007050428528088939761107053157389710902",
            "19540430494807482326159819597004422086093766032135589407132600596362845576832",
            "14940766826517323942636479241147756311199852622225275649687664389641784935947",
            "4419234939496763621076330863786513495701855246241724391626358375488475697872",
            "9088801421649573101014283686030284801466796108869023335878462724291607593530",
            "10359452186428527605436343203440067497552205259388878191021578220384701716497",
            "3478517300119284901893091970156912948790432420133812234316178878452092729974",
            "6837567842312086091520287814181175430087169027974246751610506942214842701774",
            "3161067157621608152362653341354432744960400845131437947728257924963983317266",
            "1120550406532664055539694724667294622065367841900378087843176726913374367458",
            "4158865282786404163413953114870269622875596290766033564087307867933865333818",
            "197302210312744933010843010704445784068657690384188106020011018676818793232",
            "20619701001583904760601357484951574588621083236087856586626117568842480512645",
            "20402931748843538985151001264530049874871572933694634836567070693966133783803",
            "421743594562400382753388642386256516545992082196004333756405989743524594615",
            "12650941915662020058015862023665998998969191525479888727406889100124684769509",
            "11699596668367776675346610687704220591435078791727316319397053191800576917728",
            "15549849457946371566896172786938980432421851627449396898353380550861104573629",
            "17220337697351015657950521176323262483320249231368149235373741788599650842711",
            "13536764371732269273912573961853310557438878140379554347802702086337840854307",
            "12143866164239048021030917283424216263377309185099704096317235600302831912062",
            "934650972362265999028062457054462628285482693704334323590406443310927365533",
            "5709868443893258075976348696661355716898495876243883251619397131511003808859",
            "19200870435978225707111062059747084165650991997241425080699860725083300967194",
            "7419588552507395652481651088034484897579724952953562618697845598160172257810",
            "2082940218526944230311718225077035922214683169814847712455127909555749686340",
            "19103219067921713944291392827692070036145651957329286315305642004821462161904",
        ];
        data.iter()
            .map(|each| Fr::from_str(each).unwrap())
            .collect()
    }

    /// helper function to get g1 points
    pub fn get_g1_points(&self) -> Vec<G1Affine> {
        self.g1.to_vec()
    }

    /// obtain copy of g2 points
    pub fn get_g2_points(&self) -> Vec<G2Affine> {
        self.g2.to_vec()
    }

    /// commit the actual polynomial with the values setup
    pub fn commit(&self, polynomial: &Polynomial) -> Result<G1Affine, KzgError> {
        if polynomial.len() > self.g1.len() {
            return Err(KzgError::SerializationError(
                "polynomial length is not correct".to_string(),
            ));
        }

        // Perform the multi-exponentiation
        let bases = self.g1_ifft(polynomial.len()).unwrap();
        match G1Projective::msm(&bases, &polynomial.to_vec()) {
            Ok(res) => Ok(res.into_affine()),
            Err(err) => Err(KzgError::CommitError(err.to_string())),
        }
    }

    /// 4844 compatible helper function
    pub fn blob_to_kzg_commitment(&self, blob: &Blob) -> Result<G1Affine, KzgError> {
        let polynomial = blob
            .to_polynomial()
            .map_err(|err| KzgError::SerializationError(err.to_string()))?;
        let commitment = self.commit(&polynomial)?;
        Ok(commitment)
    }

    /// helper function to work with the library and the env of the kzg instance
    pub fn compute_kzg_proof_with_roots_of_unity(
        &self,
        polynomial: &Polynomial,
        index: u64,
    ) -> Result<G1Affine, KzgError> {
        self.compute_kzg_proof(polynomial, index, &self.expanded_roots_of_unity)
    }

    /// function to compute the kzg proof given the values.
    pub fn compute_kzg_proof(
        &self,
        polynomial: &Polynomial,
        index: u64,
        root_of_unities: &Vec<Fr>,
    ) -> Result<G1Affine, KzgError> {
        if !self.params.completed_setup {
            return Err(KzgError::GenericError(
                "setup is not complete, run the data_setup functions".to_string(),
            ));
        }

        if polynomial.len() != root_of_unities.len() {
            return Err(KzgError::GenericError(
                "inconsistent length between blob and root of unities".to_string(),
            ));
        }

        let eval_fr = polynomial.to_vec();
        let mut poly_shift: Vec<Fr> = Vec::with_capacity(eval_fr.len());
        let usized_index = if let Some(x) = index.to_usize() {
            x
        } else {
            return Err(KzgError::SerializationError(
                "index couldn't be converted to usize".to_string(),
            ));
        };

        let value_fr = eval_fr[usized_index];
        let z_fr = root_of_unities[usized_index];

        for i in 0..eval_fr.len() {
            poly_shift.push(eval_fr[i] - value_fr);
        }

        let mut denom_poly = Vec::<Fr>::with_capacity(root_of_unities.len());
        for i in 0..eval_fr.len() {
            denom_poly.push(root_of_unities[i] - z_fr);
        }

        let mut quotient_poly = Vec::<Fr>::with_capacity(root_of_unities.len());

        for i in 0..root_of_unities.len() {
            if denom_poly[i].is_zero() {
                quotient_poly.push(self.compute_quotient_eval_on_domain(
                    z_fr,
                    &eval_fr,
                    value_fr,
                    &root_of_unities,
                ));
            } else {
                quotient_poly.push(poly_shift[i].div(denom_poly[i]));
            }
        }

        let g1_lagrange = self.g1_ifft(polynomial.len())?;

        match G1Projective::msm(&g1_lagrange, &quotient_poly) {
            Ok(res) => Ok(G1Affine::from(res)),
            Err(err) => Err(KzgError::SerializationError(err.to_string())),
        }
    }

    /// refer to DA for more context
    fn compute_quotient_eval_on_domain(
        &self,
        z_fr: Fr,
        eval_fr: &Vec<Fr>,
        value_fr: Fr,
        roots_of_unities: &Vec<Fr>,
    ) -> Fr {
        let mut quotient = Fr::zero();
        let mut fi = Fr::zero();
        let mut numerator: Fr = Fr::zero();
        let mut denominator: Fr = Fr::zero();
        let mut temp: Fr = Fr::zero();

        for i in 0..roots_of_unities.len() {
            let omega_i = roots_of_unities[i];
            if omega_i == z_fr {
                continue;
            }
            fi = eval_fr[i] - value_fr;
            numerator = fi.mul(omega_i);
            denominator = z_fr - omega_i;
            denominator = denominator * z_fr;
            temp = numerator.div(denominator);
            quotient = quotient + temp;
        }
        quotient
    }

    /// function to compute the inverse FFT
    pub fn g1_ifft(&self, length: usize) -> Result<Vec<G1Affine>, KzgError> {
        // is not power of 2
        if !length.is_power_of_two() {
            return Err(KzgError::FftError(
                "length provided is not a power of 2".to_string(),
            ));
        }

        let domain = GeneralEvaluationDomain::<Fr>::new(length)
            .expect("Failed to construct domain for IFFT");
        let points_projective: Vec<G1Projective> = self.g1[..length]
            .iter()
            .map(|&p| G1Projective::from(p))
            .collect();

        // Perform the IFFT
        let ifft_result = domain.ifft(&points_projective);
        let ifft_result_affine: Vec<_> = ifft_result.iter().map(|p| p.into_affine()).collect();
        Ok(ifft_result_affine)
    }

    pub fn verify_kzg_proof(
        &self,
        commitment: G1Affine,
        proof: G1Affine,
        value_fr: Fr,
        z_fr: Fr,
    ) -> bool {
        let g2_tau = if self.g2.len() > 28 {
            self.g2.get(1).unwrap().clone()
        } else {
            self.g2.get(0).unwrap().clone()
        };
        let value_g1 = (G1Affine::generator() * value_fr).into_affine();
        let commit_minus_value = (commitment - value_g1).into_affine();
        let z_g2 = (G2Affine::generator() * z_fr).into_affine();
        let x_minus_z = (g2_tau - z_g2).into_affine();
        Self::pairings_verify(commit_minus_value, G2Affine::generator(), proof, x_minus_z)
    }

    fn pairings_verify(a1: G1Affine, a2: G2Affine, b1: G1Affine, b2: G2Affine) -> bool {
        let neg_b1 = -b1;
        let p = [a1, neg_b1];
        let q = [a2, b2];
        let result = Bn254::multi_pairing(p, q);
        result.is_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use std::env;

    // Function to determine the setup based on an environment variable
    fn determine_setup() -> Kzg {
        match env::var("KZG_ENV") {
            Ok(val) if val == "mainnet-data" => Kzg::setup(false).unwrap(),
            _ => Kzg::setup(true).unwrap(),
        }
    }

    // Define a static variable for setup
    lazy_static! {
        static ref KZG_INSTANCE: Kzg = determine_setup();
        static ref KZG_3000: Kzg = Kzg::setup(true).unwrap();
    }

    #[test]
    fn test_commit_errors() {
        let mut poly = vec![];
        for _ in 0..4000 {
            poly.push(Fr::one());
        }

        let polynomial = Polynomial::new(&poly, 2).unwrap();
        let result = KZG_3000.commit(&polynomial);
        assert_eq!(
            result,
            Err(KzgError::SerializationError(
                "polynomial length is not correct".to_string()
            ))
        );
    }

    #[test]
    fn test_blob_to_kzg_commitment() {
        use crate::consts::GETTYSBURG_ADDRESS_BYTES;
        use ark_bn254::Fq;

        let blob = Blob::from_bytes_and_pad(GETTYSBURG_ADDRESS_BYTES);
        let fn_output = KZG_3000.blob_to_kzg_commitment(&blob).unwrap();
        let commitment_from_da = G1Affine::new_unchecked(
            Fq::from_str(
                "2961155957874067312593973807786254905069537311739090798303675273531563528369",
            )
            .unwrap(),
            Fq::from_str(
                "159565752702690920280451512738307422982252330088949702406468210607852362941",
            )
            .unwrap(),
        );
        assert_eq!(commitment_from_da, fn_output);
    }

    #[test]
    fn test_compute_kzg_proof() {
        use crate::consts::GETTYSBURG_ADDRESS_BYTES;
        use rand::Rng;

        let mut kzg = KZG_INSTANCE.clone();

        let input = Blob::from_bytes_and_pad(GETTYSBURG_ADDRESS_BYTES);
        let input_poly = input.to_polynomial().unwrap();

        for index in 0..input_poly.len() - 1 {
            // let index = rand::thread_rng().gen_range(0..input_poly.len());
            kzg.data_setup_custom(4, input.len().try_into().unwrap())
                .unwrap();
            let mut rand_index = rand::thread_rng().gen_range(0..kzg.expanded_roots_of_unity.len());
            loop {
                if index == rand_index {
                    rand_index = rand::thread_rng().gen_range(0..kzg.expanded_roots_of_unity.len());
                } else {
                    break;
                }
            }
            let commitment = kzg.commit(&input_poly.clone()).unwrap();
            let proof = kzg
                .compute_kzg_proof_with_roots_of_unity(&input_poly, index.try_into().unwrap())
                .unwrap();
            let value_fr = input_poly.get_at_index(index).unwrap();
            let z_fr = kzg.get_nth_root_of_unity(index).unwrap();
            let pairing_result =
                kzg.verify_kzg_proof(commitment, proof, value_fr.clone(), z_fr.clone());
            assert_eq!(pairing_result, true);
            assert_eq!(
                kzg.verify_kzg_proof(
                    commitment,
                    proof,
                    value_fr.clone(),
                    kzg.get_nth_root_of_unity(rand_index).unwrap().clone()
                ),
                false
            )
        }
    }
}
