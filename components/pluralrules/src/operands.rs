use icu_num_util::FixedDecimal;
use std::convert::TryFrom;
use std::io::Error as IOError;
use std::isize;
use std::num::ParseIntError;
use std::str::FromStr;

/// A full plural operands representation of a number. See [CLDR Plural Rules](http://unicode.org/reports/tr35/tr35-numbers.html#Language_Plural_Rules) for complete operands description.
/// Plural operands in compliance with [CLDR Plural Rules](http://unicode.org/reports/tr35/tr35-numbers.html#Language_Plural_Rules).
///
/// See [full operands description](http://unicode.org/reports/tr35/tr35-numbers.html#Operands).
///
/// # Examples
///
/// From int
///
/// ```
/// use icu_pluralrules::PluralOperands;
/// assert_eq!(PluralOperands {
///    i: 2,
///    v: 0,
///    w: 0,
///    f: 0,
///    t: 0,
/// }, PluralOperands::from(2_usize))
/// ```
///
/// From float
///
/// ```
/// use std::str::FromStr;
/// use icu_pluralrules::PluralOperands;
/// assert_eq!(Ok(PluralOperands {
///    i: 1234,
///    v: 3,
///    w: 3,
///    f: 567,
///    t: 567,
/// }), "-1234.567".parse())
/// ```
///
/// From &str
///
/// ```
/// use std::convert::TryFrom;
/// use icu_pluralrules::PluralOperands;
/// assert_eq!(Ok(PluralOperands {
///    i: 123,
///    v: 2,
///    w: 2,
///    f: 45,
///    t: 45,
/// }), "123.45".parse())
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PluralOperands {
    /// Integer value of input
    pub i: u64,
    /// Number of visible fraction digits with trailing zeros
    pub v: usize,
    /// Number of visible fraction digits without trailing zeros
    pub w: usize,
    /// Visible fraction digits with trailing zeros
    pub f: u64,
    /// Visible fraction digits without trailing zeros
    pub t: u64,
}

impl PluralOperands {
    /// Returns the number represented by this [PluralOperands] as floating point.
    /// The precision of the number returned is up to the representation accuracy
    /// of a double.
    pub fn n(&self) -> f64 {
        let fraction = self.t as f64 / 10_f64.powi(self.v as i32);
        self.i as f64 + fraction
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum OperandsError {
    /// Input to the Operands parsing was empty.
    Empty,
    /// Input to the Operands parsing was invalid.
    Invalid,
}

impl From<ParseIntError> for OperandsError {
    fn from(_: ParseIntError) -> Self {
        Self::Invalid
    }
}

impl From<IOError> for OperandsError {
    fn from(_: IOError) -> Self {
        Self::Invalid
    }
}

impl FromStr for PluralOperands {
    type Err = OperandsError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(OperandsError::Empty);
        }

        let abs_str = if input.starts_with('-') {
            &input[1..]
        } else {
            &input
        };

        let (
            integer_digits,
            num_fraction_digits0,
            num_fraction_digits,
            fraction_digits0,
            fraction_digits,
        ) = if let Some(sep_idx) = abs_str.find('.') {
            let int_str = &abs_str[..sep_idx];
            let dec_str = &abs_str[(sep_idx + 1)..];

            let integer_digits = u64::from_str(&int_str)?;

            let dec_str_no_zeros = dec_str.trim_end_matches('0');

            let num_fraction_digits0 = dec_str.len() as usize;
            let num_fraction_digits = dec_str_no_zeros.len() as usize;

            let fraction_digits0 = u64::from_str(&dec_str)?;
            let fraction_digits =
                if num_fraction_digits == 0 || num_fraction_digits == num_fraction_digits0 {
                    fraction_digits0
                } else {
                    u64::from_str(&dec_str_no_zeros)?
                };

            (
                integer_digits,
                num_fraction_digits0,
                num_fraction_digits,
                fraction_digits0,
                fraction_digits,
            )
        } else {
            let integer_digits = u64::from_str(&abs_str)?;
            (integer_digits, 0, 0, 0, 0)
        };

        Ok(PluralOperands {
            i: integer_digits,
            v: num_fraction_digits0,
            w: num_fraction_digits,
            f: fraction_digits0,
            t: fraction_digits,
        })
    }
}

macro_rules! impl_integer_type {
    ($ty:ident) => {
        impl From<$ty> for PluralOperands {
            fn from(input: $ty) -> Self {
                PluralOperands {
                    i: input as u64,
                    v: 0,
                    w: 0,
                    f: 0,
                    t: 0,
                }
            }
        }
    };
    ($($ty:ident)+) => {
        $(impl_integer_type!($ty);)+
    };
}

macro_rules! impl_signed_integer_type {
    ($ty:ident) => {
        impl TryFrom<$ty> for PluralOperands {
            type Error = OperandsError;
            fn try_from(input: $ty) -> Result<Self, Self::Error> {
                let x = input.checked_abs().ok_or(OperandsError::Invalid)?;
                Ok(PluralOperands {
                    i: x as u64,
                    v: 0,
                    w: 0,
                    f: 0,
                    t: 0,
                })
            }
        }
    };
    ($($ty:ident)+) => {
        $(impl_signed_integer_type!($ty);)+
    };
}

impl_integer_type!(u8 u16 u32 u64 u128 usize);
impl_signed_integer_type!(i8 i16 i32 i64 i128 isize);

impl From<&FixedDecimal> for PluralOperands {
    /// Converts a [icu_num_util::FixedDecimal] to [PluralOperands]
    fn from(f: &FixedDecimal) -> Self {
        let mag_range = f.magnitude_range();
        let mag_high = *mag_range.end();
        let mag_low = *mag_range.start();

        let mut integer_part: u64 = 0;
        let integer_range = 0..=mag_high;
        for magnitude in integer_range.rev() {
            integer_part *= 10;
            integer_part += f.digit_at(magnitude) as u64;
        }

        // The fractional part of the number, expressed as an integer to preserve trailing zeroes.
        // For example, could be "100" if the stored number is "0.100".
        let mut fraction_part_full: u64 = 0;
        // A running total of the number of trailing zeros seen in the fractional part of the
        // number.
        let mut num_trailing_zeros = 0;
        // 10^num_trailing_zeros.
        let mut weight_trailing_zeros = 1;
        let fractional_magnitude_range = mag_low..=-1;
        for magnitude in fractional_magnitude_range.rev() {
            let digit = f.digit_at(magnitude);
            if digit == 0 {
                num_trailing_zeros += 1;
                weight_trailing_zeros *= 10;
            } else {
                num_trailing_zeros = 0;
                weight_trailing_zeros = 1;
            }
            fraction_part_full *= 10;
            fraction_part_full += digit as u64;
        }
        let fraction_part_nozeros = fraction_part_full / weight_trailing_zeros;
        let num_fractional_digits = (-std::cmp::min(0, mag_low)) as usize;

        PluralOperands {
            i: integer_part,
            v: num_fractional_digits,
            w: num_fractional_digits - num_trailing_zeros,
            f: fraction_part_full,
            t: fraction_part_nozeros,
        }
    }
}
