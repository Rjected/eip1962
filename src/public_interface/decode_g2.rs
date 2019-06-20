use crate::weierstrass::twist;
use crate::weierstrass::cubic_twist;
use crate::field::{SizedPrimeField};
use crate::fp::Fp;
use crate::extension_towers::fp2;
use crate::extension_towers::fp3;
use crate::representation::ElementRepr;
use crate::pairings::{frobenius_calculator_fp2, frobenius_calculator_fp3};
use crate::traits::FieldElement;

use num_bigint::BigUint;

use super::decode_fp::*;
use super::constants::*;

use crate::errors::ApiError;

pub(crate) fn create_fp2_extension<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>,
    >
    (
        bytes: &'a [u8], 
        field_byte_len: usize,
        base_field: &'a F,
    ) -> Result<(fp2::Extension2<'a, FE, F>, &'a [u8]), ApiError>
{
    if bytes.len() < EXTENSION_DEGREE_ENCODING_LENGTH {
        return Err(ApiError::InputError("Input is not long enough to get extension degree".to_owned()));
    }
    let (extension_degree, rest) = bytes.split_at(EXTENSION_DEGREE_ENCODING_LENGTH);
    if extension_degree[0] != EXTENSION_DEGREE_2 {
        return Err(ApiError::UnknownParameter("Extension degree expected to be 2".to_owned()));
    }

    let (fp_non_residue, rest): (Fp<'a, FE, F>, _) = decode_fp(&rest, field_byte_len, base_field)?;
    if fp_non_residue.is_zero() {
        return Err(ApiError::UnexpectedZero("Fp2 non-residue can not be zero".to_owned()));
    }
    let mut extension_2 = fp2::Extension2 {
        field: base_field,
        non_residue: fp_non_residue,
        frobenius_coeffs_c1: [Fp::zero(base_field), Fp::zero(base_field)]
    };

    let coeffs = frobenius_calculator_fp2(&extension_2).map_err(|_| {
        ApiError::UnknownParameter("Failed to calculate Frobenius coeffs for Fp2".to_owned())
    })?;
    extension_2.frobenius_coeffs_c1 = coeffs;
    
    Ok((extension_2, rest))
}

pub(crate) fn create_fp3_extension<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>,
    >
    (
        bytes: &'a [u8], 
        modulus: BigUint,
        field_byte_len: usize,
        base_field: &'a F,
    ) -> Result<(fp3::Extension3<'a, FE, F>, &'a [u8]), ApiError>
{
    if bytes.len() < EXTENSION_DEGREE_ENCODING_LENGTH {
        return Err(ApiError::InputError("Input is not long enough to get extension degree".to_owned()));
    }
    let (extension_degree, rest) = bytes.split_at(EXTENSION_DEGREE_ENCODING_LENGTH);
    if extension_degree[0] != EXTENSION_DEGREE_3 {
        return Err(ApiError::UnknownParameter("Extension degree expected to be 3".to_owned()));
    }

    let (fp_non_residue, rest): (Fp<'a, FE, F>, _) = decode_fp(&rest, field_byte_len, base_field)?;
    if fp_non_residue.is_zero() {
        return Err(ApiError::UnexpectedZero("Fp2 non-residue can not be zero".to_owned()));
    }
    let mut extension_3 = fp3::Extension3 {
        field: base_field,
        non_residue: fp_non_residue,
        frobenius_coeffs_c1: [Fp::zero(base_field), Fp::zero(base_field), Fp::zero(base_field)],
        frobenius_coeffs_c2: [Fp::zero(base_field), Fp::zero(base_field), Fp::zero(base_field)]
    };

    let (coeffs_1, coeffs_2) = frobenius_calculator_fp3(modulus, &extension_3).map_err(|_| {
        ApiError::UnknownParameter("Failed to calculate Frobenius coeffs for Fpe".to_owned())
    })?;
    extension_3.frobenius_coeffs_c1 = coeffs_1;
    extension_3.frobenius_coeffs_c2 = coeffs_2;
    
    Ok((extension_3, rest))
}

pub(crate) fn decode_g2_point_from_xy_in_fp2<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>,
    GE: ElementRepr,
    G: SizedPrimeField<Repr = GE>
    >
    (
        bytes: &'a [u8], 
        field_byte_len: usize,
        curve: &'a twist::WeierstrassCurveTwist<'a, FE, F, GE, G>
    ) -> Result<(twist::TwistPoint<'a, FE, F, GE, G>, &'a [u8]), ApiError>
{
    let (x, rest) = decode_fp2(&bytes, field_byte_len, curve.base_field)?;
    let (y, rest) = decode_fp2(&rest, field_byte_len, curve.base_field)?;
    
    let p: twist::TwistPoint<'a, FE, F, GE, G> = twist::TwistPoint::point_from_xy(&curve, x, y);
    
    Ok((p, rest))
}

pub(crate) fn decode_g2_point_from_xy_in_fp3<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>,
    GE: ElementRepr,
    G: SizedPrimeField<Repr = GE>
    >
    (
        bytes: &'a [u8], 
        field_byte_len: usize,
        curve: &'a cubic_twist::WeierstrassCurveTwist<'a, FE, F, GE, G>
    ) -> Result<(cubic_twist::TwistPoint<'a, FE, F, GE, G>, &'a [u8]), ApiError>
{
    let (x, rest) = decode_fp3(&bytes, field_byte_len, curve.base_field)?;
    let (y, rest) = decode_fp3(&rest, field_byte_len, curve.base_field)?;
    
    let p: cubic_twist::TwistPoint<'a, FE, F, GE, G> = cubic_twist::TwistPoint::point_from_xy(&curve, x, y);
    
    Ok((p, rest))
}

pub(crate) fn serialize_g2_point_in_fp2<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>,
    GE: ElementRepr,
    G: SizedPrimeField<Repr = GE>
    >
    (
        modulus_len: usize,
        point: &twist::TwistPoint<'a, FE, F, GE, G>
    ) -> Result<Vec<u8>, ApiError>
{
    let (x, y) = point.into_xy();
    let mut result = serialize_fp2_fixed_len(modulus_len, &x)?;
    result.extend(serialize_fp2_fixed_len(modulus_len, &y)?);
    
    Ok(result)
}