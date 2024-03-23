extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::CStr;
use molecule::prelude::Entity;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

use crate::schema::dob_721::{TraitTypeUnion, TraitsBase};

#[repr(u64)]
pub enum Error {
    UnexpectedArgCount = 1,
    UnexpectedArgBytesLength,
    InvalidArgFormat,
    InvalidArgMolFormat,
    InsufficientDNABytes,
    InvalidDNASetSchema,
    InvalidDNAByteLengthSchema,
    InvalidDNARangeSchema,
}

#[derive(serde::Serialize)]
pub enum Trait {
    String(String),
    Number(u64),
    Float(f64),
}

#[derive(serde::Serialize, Default)]
pub struct DNA {
    pub name: String,
    pub traits: Vec<Trait>,
}

// example:
// argv[1] = efc2866a311da5b6dfcdfc4e3c22d00d024a53217ebc33855eeab1068990ed9d (hexed DNA string in Spore)
// argv[2] = 1250945 (block number while minting)
// argv[3] = d48869363ff41a103b131a29f43...d7be6eeaf513c2c3ae056b9b8c2e1 (traits config in Cluster)
pub fn dobs_decode(argc: u64, argv: *const *const i8) -> Result<Vec<u8>, Error> {
    if argc != 4 {
        return Err(Error::UnexpectedArgCount);
    }
    let mut params = Vec::new();
    for i in 1..argc {
        let argn = unsafe { CStr::from_ptr(argv.add(i as usize).read()) };
        params.push(argn.to_bytes().to_vec());
    }

    ///////////////////////////
    // decoder logic start
    ///////////////////////////

    // parse decoder parameters
    let mut dna = {
        let value = &params[0];
        if value.len() % 2 != 0 {
            return Err(Error::UnexpectedArgBytesLength);
        }
        let mut dna = Vec::with_capacity(value.len() / 2);
        faster_hex::hex_decode(&value, &mut dna).map_err(|_| Error::InvalidArgFormat)?;
        dna
    };
    let mut rng = {
        let value = String::from_utf8_lossy(&params[1]);
        let block_number = u64::from_str_radix(&value, 10).map_err(|_| Error::InvalidArgFormat)?;
        SmallRng::seed_from_u64(block_number)
    };
    let traits_base = {
        let value = &params[2];
        if value.len() % 2 != 0 {
            return Err(Error::UnexpectedArgBytesLength);
        }
        let mut traits = Vec::with_capacity(value.len() / 2);
        faster_hex::hex_decode(&value, &mut traits).map_err(|_| Error::InvalidArgFormat)?;
        TraitsBase::from_compatible_slice(&traits).map_err(|_| Error::InvalidArgMolFormat)?
    };

    // decode DNA from traits base
    let mut result = Vec::new();
    for trait_schema in traits_base.into_iter() {
        let mut parsed_dna = DNA::default();
        parsed_dna.name = trait_schema.name().into();
        for schema in trait_schema.traits().into_iter() {
            let byte_length = schema.byte_length().into();
            let mut dna_segment = Vec::new();
            for _ in 0..byte_length {
                if dna.is_empty() {
                    return Err(Error::InsufficientDNABytes);
                }
                dna_segment.push(dna.remove(0));
            }
            let offset = match dna_segment.len() {
                1 => dna_segment[0] as u64,
                2 => u16::from_le_bytes(dna_segment.try_into().unwrap()) as u64,
                4 => u32::from_be_bytes(dna_segment.try_into().unwrap()) as u64,
                8 => u64::from_le_bytes(dna_segment.try_into().unwrap()),
                _ => return Err(Error::InvalidDNAByteLengthSchema),
            };
            let Some(traits_pool) = schema.traits_pool().to_opt() else {
                parsed_dna.traits.push(Trait::Number(offset));
                continue;
            };
            for trait_type in traits_pool.into_iter() {
                match trait_type.to_enum() {
                    TraitTypeUnion::StringVec(strings) => {
                        if strings.is_empty() {
                            return Err(Error::InvalidDNASetSchema);
                        }
                        let offset = offset as usize % strings.len();
                        let value = strings.get_unchecked(offset).into();
                        parsed_dna.traits.push(Trait::String(value));
                    }
                    TraitTypeUnion::NumberVec(numbers) => {
                        if numbers.is_empty() {
                            return Err(Error::InvalidDNASetSchema);
                        }
                        let offset = offset as usize % numbers.len();
                        let value = numbers.get_unchecked(offset);
                        parsed_dna.traits.push(Trait::Number(value.into()));
                    }
                    TraitTypeUnion::FloatVec(floats) => {
                        if floats.numerator_vec().is_empty() {
                            return Err(Error::InvalidDNASetSchema);
                        }
                        let offset = offset as usize % floats.numerator_vec().len();
                        let numerator: u64 = floats.numerator_vec().get_unchecked(offset).into();
                        let denominator: u64 = floats.denominator().into();
                        parsed_dna
                            .traits
                            .push(Trait::Float(numerator as f64 / denominator as f64));
                    }
                    TraitTypeUnion::MutantVec(mutant_numbers) => {
                        if mutant_numbers.is_empty() {
                            return Err(Error::InvalidDNASetSchema);
                        }
                        let offset = (offset + rng.next_u64()) as usize % mutant_numbers.len();
                        let value = mutant_numbers.get_unchecked(offset);
                        parsed_dna.traits.push(Trait::Number(value.into()));
                    }
                    TraitTypeUnion::NumberRange(number_range) => {
                        let upperbound: u64 = number_range.nth1().into();
                        let lowerbound: u64 = number_range.nth0().into();
                        if upperbound < lowerbound {
                            return Err(Error::InvalidDNARangeSchema);
                        }
                        let offset = offset % (upperbound - lowerbound);
                        parsed_dna.traits.push(Trait::Number(lowerbound + offset));
                    }
                    TraitTypeUnion::FloatRange(float_range) => {
                        let upperbound: u64 = float_range.numerator_range().nth1().into();
                        let lowerbound: u64 = float_range.numerator_range().nth0().into();
                        if upperbound < lowerbound {
                            return Err(Error::InvalidDNARangeSchema);
                        }
                        let offset = offset % (upperbound - lowerbound);
                        let numerator = lowerbound + offset;
                        let denominator: u64 = float_range.denominator().into();
                        parsed_dna
                            .traits
                            .push(Trait::Float(numerator as f64 / denominator as f64));
                    }
                    TraitTypeUnion::MutantRange(mutant_number_range) => {
                        let upperbound: u64 = mutant_number_range.nth1().into();
                        let lowerbound: u64 = mutant_number_range.nth0().into();
                        if upperbound < lowerbound {
                            return Err(Error::InvalidDNARangeSchema);
                        }
                        let offset = (offset + rng.next_u64()) % (upperbound - lowerbound);
                        parsed_dna.traits.push(Trait::Number(lowerbound + offset));
                    }
                }
            }
        }
        result.push(parsed_dna);
    }

    ///////////////////////////
    // decoder logic end
    ///////////////////////////

    Ok(serde_json::to_string(&result).unwrap().into_bytes())
}
