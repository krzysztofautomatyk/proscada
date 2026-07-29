//! Encoding and decoding of multi-register Modbus values.
//!
//! Modbus only defines 16-bit registers; every wider type is a vendor
//! convention layered on top. ProScada supports the two orders that cover
//! virtually all PLCs: high word first (the Modbus norm) and low word first
//! (common on Schneider/Wago-style devices). Byte order inside a register is
//! always big-endian, as mandated by the protocol.

use crate::project::{TagDataType, WordOrder};

/// Reorder raw registers into high-word-first order for decoding.
fn normalize(words: &[u16], order: WordOrder) -> Vec<u16> {
    match order {
        WordOrder::HighWordFirst => words.to_vec(),
        WordOrder::LowWordFirst => words.iter().rev().copied().collect(),
    }
}

/// Convert a high-word-first buffer back to the wire order.
fn denormalize(words: Vec<u16>, order: WordOrder) -> Vec<u16> {
    match order {
        WordOrder::HighWordFirst => words,
        WordOrder::LowWordFirst => words.into_iter().rev().collect(),
    }
}

fn to_u64(words: &[u16]) -> u64 {
    words
        .iter()
        .fold(0_u64, |acc, w| (acc << 16) | u64::from(*w))
}

fn from_u64(value: u64, count: usize) -> Vec<u16> {
    (0..count)
        .map(|i| ((value >> (16 * (count - 1 - i))) & 0xFFFF) as u16)
        .collect()
}

/// Decode `words` into the engineering-unit value of `data_type`.
///
/// Returns `None` when the slice is shorter than the type requires, which the
/// caller must treat as "no fresh value" rather than as zero.
pub fn decode(data_type: TagDataType, order: WordOrder, words: &[u16]) -> Option<f64> {
    let count = usize::from(data_type.register_count()?);
    if words.len() < count {
        return None;
    }
    let ordered = normalize(&words[..count], order);
    let raw = to_u64(&ordered);
    let value = match data_type {
        TagDataType::Bool => f64::from(u16::from(raw != 0)),
        TagDataType::U16 => f64::from(raw as u16),
        TagDataType::I16 => f64::from(raw as u16 as i16),
        TagDataType::U32 => f64::from(raw as u32),
        TagDataType::I32 => f64::from(raw as u32 as i32),
        TagDataType::F32 => f64::from(f32::from_bits(raw as u32)),
        TagDataType::U64 => raw as f64,
        TagDataType::I64 => raw as i64 as f64,
        TagDataType::F64 => f64::from_bits(raw),
        TagDataType::String => return None,
    };
    Some(value)
}

/// Encode an engineering-unit value into wire-order registers.
///
/// Values outside the representable range of the target type are rejected
/// instead of being clamped, because a silently clamped setpoint is a process
/// hazard.
pub fn encode(data_type: TagDataType, order: WordOrder, value: f64) -> Result<Vec<u16>, String> {
    if !value.is_finite() {
        return Err("Write value must be finite".into());
    }
    let count = usize::from(
        data_type
            .register_count()
            .ok_or("Data type has no fixed Modbus width")?,
    );
    let raw: u64 = match data_type {
        TagDataType::Bool => u64::from(value != 0.0),
        TagDataType::U16 => range_checked(value, 0.0, f64::from(u16::MAX), "u16")? as u64,
        TagDataType::I16 => {
            (range_checked(value, f64::from(i16::MIN), f64::from(i16::MAX), "i16")? as i16) as u16
                as u64
        }
        TagDataType::U32 => range_checked(value, 0.0, f64::from(u32::MAX), "u32")? as u64,
        TagDataType::I32 => {
            (range_checked(value, f64::from(i32::MIN), f64::from(i32::MAX), "i32")? as i32) as u32
                as u64
        }
        TagDataType::F32 => {
            let narrowed = value as f32;
            if !narrowed.is_finite() {
                return Err(format!("Value {value} is not representable as f32"));
            }
            u64::from(narrowed.to_bits())
        }
        TagDataType::U64 => {
            let rounded = value.round();
            // `u64::MAX as f64` rounds up to 2^64. Treat that exclusive
            // boundary explicitly so a value of 2^64 is rejected rather than
            // saturating during the cast.
            if !(0.0..18_446_744_073_709_551_616.0).contains(&rounded) {
                return Err(format!("Value {value} is out of range for u64"));
            }
            rounded as u64
        }
        TagDataType::I64 => {
            let rounded = value.round();
            // `i64::MAX as f64` is exactly 2^63, which is outside i64.
            if !(-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&rounded) {
                return Err(format!("Value {value} is out of range for i64"));
            }
            (rounded as i64) as u64
        }
        TagDataType::F64 => value.to_bits(),
        TagDataType::String => return Err("String tags cannot be written over Modbus".into()),
    };
    Ok(denormalize(from_u64(raw, count), order))
}

fn range_checked(value: f64, min: f64, max: f64, label: &str) -> Result<f64, String> {
    let rounded = value.round();
    if rounded < min || rounded > max {
        return Err(format!(
            "Value {value} is out of range for {label} ({min}..={max})"
        ));
    }
    Ok(rounded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_roundtrip_high_word_first() {
        let words = encode(TagDataType::F32, WordOrder::HighWordFirst, 12.5).expect("encode");
        assert_eq!(words.len(), 2);
        let back = decode(TagDataType::F32, WordOrder::HighWordFirst, &words).expect("decode");
        assert!((back - 12.5).abs() < f64::EPSILON);
    }

    #[test]
    fn word_order_actually_swaps_registers() {
        let hi = encode(
            TagDataType::U32,
            WordOrder::HighWordFirst,
            0x1234_5678_u32 as f64,
        )
        .expect("encode");
        let lo = encode(
            TagDataType::U32,
            WordOrder::LowWordFirst,
            0x1234_5678_u32 as f64,
        )
        .expect("encode");
        assert_eq!(hi, vec![0x1234, 0x5678]);
        assert_eq!(lo, vec![0x5678, 0x1234]);
        assert_eq!(
            decode(TagDataType::U32, WordOrder::LowWordFirst, &lo),
            decode(TagDataType::U32, WordOrder::HighWordFirst, &hi)
        );
    }

    #[test]
    fn signed_types_decode_negative_values() {
        assert_eq!(
            decode(TagDataType::I16, WordOrder::HighWordFirst, &[0xFFFF]),
            Some(-1.0)
        );
        assert_eq!(
            decode(
                TagDataType::I32,
                WordOrder::HighWordFirst,
                &[0xFFFF, 0xFFFE]
            ),
            Some(-2.0)
        );
    }

    #[test]
    fn sixty_four_bit_types_use_four_registers() {
        let words = encode(TagDataType::F64, WordOrder::HighWordFirst, -1234.5).expect("encode");
        assert_eq!(words.len(), 4);
        assert_eq!(
            decode(TagDataType::F64, WordOrder::HighWordFirst, &words),
            Some(-1234.5)
        );
    }

    #[test]
    fn out_of_range_values_are_rejected_not_clamped() {
        assert!(encode(TagDataType::U16, WordOrder::HighWordFirst, -1.0).is_err());
        assert!(encode(TagDataType::U16, WordOrder::HighWordFirst, 65_536.0).is_err());
        assert!(encode(TagDataType::I16, WordOrder::HighWordFirst, 40_000.0).is_err());
        assert!(encode(TagDataType::F32, WordOrder::HighWordFirst, f64::NAN).is_err());
        assert!(encode(
            TagDataType::U64,
            WordOrder::HighWordFirst,
            18_446_744_073_709_551_616.0
        )
        .is_err());
        assert!(encode(
            TagDataType::I64,
            WordOrder::HighWordFirst,
            9_223_372_036_854_775_808.0
        )
        .is_err());
    }

    #[test]
    fn short_buffers_decode_to_none_instead_of_zero() {
        assert_eq!(
            decode(TagDataType::U32, WordOrder::HighWordFirst, &[1]),
            None
        );
        assert_eq!(
            decode(TagDataType::String, WordOrder::HighWordFirst, &[1, 2, 3, 4]),
            None
        );
    }
}
