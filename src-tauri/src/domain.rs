//! Exact financial policy. No binary floating point or SQL financial arithmetic.
use std::fmt;

pub const RULE_VERSION: &str = "twx-fixed-v1-provisional";
pub const PAYOUT_SCALE: i64 = 1_000_000;

#[derive(Debug)]
pub enum Error {
    Invalid(&'static str),
    Overflow,
    Conflict,
    Missing,
    Database(rusqlite::Error),
    Json(serde_json::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => write!(f, "{message}"),
            Self::Overflow => write!(f, "Exact numeric range exceeded"),
            Self::Conflict => write!(f, "Record changed; reload before saving"),
            Self::Missing => write!(f, "Record not found"),
            Self::Database(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
        }
    }
}
impl std::error::Error for Error {}
impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Self::Database(value)
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
pub type Result<T> = std::result::Result<T, Error>;

/// Strict unsigned decimal input; does not silently round excess precision.
pub fn parse_scaled(input: &str, decimal_places: u32) -> Result<i64> {
    if decimal_places > 6 {
        return Err(Error::Invalid("Unsupported input scale"));
    }
    let (whole, fractional) = input.split_once('.').unwrap_or((input, ""));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || (input.contains('.') && fractional.is_empty())
        || !fractional.bytes().all(|b| b.is_ascii_digit())
        || fractional.len() > decimal_places as usize
    {
        return Err(Error::Invalid(
            "Expected a nonnegative decimal at the approved precision",
        ));
    }
    let coefficient = whole.parse::<i128>().map_err(|_| Error::Overflow)?;
    let fraction = if fractional.is_empty() {
        0
    } else {
        fractional.parse::<i128>().map_err(|_| Error::Overflow)?
    };
    let value = coefficient
        .checked_mul(10_i128.pow(decimal_places))
        .and_then(|v| {
            v.checked_add(fraction * 10_i128.pow(decimal_places - fractional.len() as u32))
        })
        .ok_or(Error::Overflow)?;
    narrow(value)
}
pub fn parse_percentage(input: &str) -> Result<i64> {
    let value = parse_scaled(input, 4)?;
    payout_rate(value)?;
    Ok(value)
}
/// Informational decimal text, not an arithmetic operand or an approved fixed scale.
pub fn validate_mileage(input: Option<&str>) -> Result<()> {
    if let Some(input) = input {
        let (whole, fraction) = input.split_once('.').unwrap_or((input, ""));
        if input.len() > 128
            || whole.is_empty()
            || !whole.bytes().all(|b| b.is_ascii_digit())
            || (input.contains('.') && fraction.is_empty())
            || !fraction.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(Error::Invalid(
                "Mileage must be nonnegative decimal text (up to 128 characters)",
            ));
        }
    }
    Ok(())
}
pub fn nonnegative(value: i64) -> Result<i64> {
    if value < 0 {
        Err(Error::Invalid("Negative input is prohibited"))
    } else {
        Ok(value)
    }
}
pub fn payout_rate(value: i64) -> Result<i64> {
    if !(0..=PAYOUT_SCALE).contains(&value) {
        Err(Error::Invalid(
            "Payout percentage must be from 0% through 100%",
        ))
    } else {
        Ok(value)
    }
}
fn narrow(value: i128) -> Result<i64> {
    i64::try_from(value).map_err(|_| Error::Overflow)
}

/// Decimal half-up: nearest integer, ties away from zero. Avoids numerator + half overflow.
pub fn half_up(numerator: i128, denominator: i128) -> Result<i128> {
    if denominator <= 0 {
        return Err(Error::Invalid("Positive denominator required"));
    }
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let magnitude = remainder.checked_abs().ok_or(Error::Overflow)?;
    let threshold = denominator / 2 + denominator % 2;
    if magnitude >= threshold {
        quotient
            .checked_add(if numerator < 0 { -1 } else { 1 })
            .ok_or(Error::Overflow)
    } else {
        Ok(quotient)
    }
}
pub fn load_total(tons_milli: i64, completion_rate_scaled: i64) -> Result<i64> {
    let tons = i128::from(nonnegative(tons_milli)?);
    let rate = i128::from(nonnegative(completion_rate_scaled)?);
    narrow(half_up(
        tons.checked_mul(rate).ok_or(Error::Overflow)?,
        100_000,
    )?)
}
pub fn sum_loads(totals: &[i64]) -> Result<i64> {
    let sum = totals.iter().try_fold(0_i128, |sum, &value| {
        sum.checked_add(i128::from(nonnegative(value)?))
            .ok_or(Error::Overflow)
    })?;
    narrow(sum)
}
pub fn driver_gross(gross_route_cents: i64, payout_rate_ppm: i64) -> Result<i64> {
    let gross = i128::from(nonnegative(gross_route_cents)?);
    let percentage = i128::from(payout_rate(payout_rate_ppm)?);
    narrow(half_up(
        gross.checked_mul(percentage).ok_or(Error::Overflow)?,
        i128::from(PAYOUT_SCALE),
    )?)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Add,
    Deduct,
}
impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Add => "ADD",
            Self::Deduct => "DEDUCT",
        }
    }
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "ADD" => Ok(Self::Add),
            "DEDUCT" => Ok(Self::Deduct),
            _ => Err(Error::Invalid("Invalid adjustment direction")),
        }
    }
}
pub fn apply_adjustments(gross_cents: i64, adjustments: &[(Direction, i64)]) -> Result<i64> {
    let total =
        adjustments
            .iter()
            .try_fold(i128::from(gross_cents), |total, &(direction, amount)| {
                let amount = i128::from(nonnegative(amount)?);
                match direction {
                    Direction::Add => total.checked_add(amount),
                    Direction::Deduct => total.checked_sub(amount),
                }
                .ok_or(Error::Overflow)
            })?;
    narrow(total)
}
pub fn driver_payroll(amount_cents: i64, ach_charge_cents: i64) -> Result<i64> {
    narrow(i128::from(amount_cents) + i128::from(nonnegative(ach_charge_cents)?))
}
#[derive(Debug, PartialEq, Eq)]
pub struct SettlementTotals {
    pub gross_route_cents: i64,
    pub driver_gross_cents: i64,
    pub final_payout_cents: i64,
}
pub fn settlement(
    totals: &[i64],
    rate: i64,
    adjustments: &[(Direction, i64)],
) -> Result<SettlementTotals> {
    let gross_route_cents = sum_loads(totals)?;
    let driver_gross_cents = driver_gross(gross_route_cents, rate)?;
    let final_payout_cents = apply_adjustments(driver_gross_cents, adjustments)?;
    Ok(SettlementTotals {
        gross_route_cents,
        driver_gross_cents,
        final_payout_cents,
    })
}
pub fn validate_period(start: &str, end: &str) -> Result<()> {
    fn valid(value: &str) -> bool {
        let b = value.as_bytes();
        if b.len() != 10
            || b[4] != b'-'
            || b[7] != b'-'
            || !b
                .iter()
                .enumerate()
                .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        {
            return false;
        }
        let year = value[..4].parse::<u32>().unwrap_or(0);
        let month = value[5..7].parse::<u32>().unwrap_or(0);
        let day = value[8..].parse::<u32>().unwrap_or(0);
        let leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => 0,
        };
        year > 0 && day > 0 && day <= days
    }
    if !valid(start) || !valid(end) || end < start {
        Err(Error::Invalid("Invalid calendar date or reversed period"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fractional_loads_and_half_cent_boundaries() {
        assert_eq!(
            load_total(
                parse_scaled("12.345", 3).unwrap(),
                parse_scaled("6.7891", 4).unwrap()
            )
            .unwrap(),
            8381
        );
        assert_eq!(load_total(1, 49_999).unwrap(), 0);
        assert_eq!(load_total(1, 50_000).unwrap(), 1);
        assert_eq!(load_total(1, 50_001).unwrap(), 1);
        assert_eq!(sum_loads(&[load_total(1, 50_000).unwrap(); 2]).unwrap(), 2); // rounded sum would be 1
        assert_eq!(half_up(-5, 10).unwrap(), -1);
        assert_eq!(half_up(-4, 10).unwrap(), 0);
        assert_eq!(half_up(i128::MAX, 1).unwrap(), i128::MAX);
        assert_eq!(half_up(i128::MIN, 1).unwrap(), i128::MIN);
        assert!(half_up(1, 0).is_err());
    }
    #[test]
    fn exact_input_parsing_and_bounds() {
        assert_eq!(parse_scaled("9223372036854775.807", 3).unwrap(), i64::MAX);
        assert_eq!(parse_scaled("922337203685477.5807", 4).unwrap(), i64::MAX);
        assert_eq!(parse_scaled("92233720368547758.07", 2).unwrap(), i64::MAX);
        for value in [
            "-1",
            "NaN",
            "inf",
            "1e3",
            " 1",
            "1 ",
            "1.",
            ".5",
            "1.2.3",
            "0.0001",
            "9223372036854775.808",
        ] {
            assert!(parse_scaled(value, 3).is_err(), "{value}");
        }
        assert!(parse_scaled("9".repeat(100).as_str(), 4).is_err());
        assert!(load_total(-1, 1).is_err());
        assert!(load_total(1, -1).is_err());
        assert!(load_total(i64::MAX, i64::MAX).is_err());
        assert_eq!(load_total(i64::MAX, 100_000).unwrap(), i64::MAX);
        assert_eq!(load_total(0, i64::MAX).unwrap(), 0);
        assert!(sum_loads(&[i64::MAX, 1]).is_err());
        assert!(driver_payroll(i64::MAX, 1).is_err());
        assert!(apply_adjustments(i64::MIN, &[(Direction::Deduct, 1)]).is_err());
    }
    #[test]
    fn percentage_rounding_adjustments_and_ach() {
        assert_eq!(parse_percentage("85.1234").unwrap(), 851234);
        assert!(parse_percentage("100.0001").is_err());
        assert!(parse_percentage("1.00001").is_err());
        assert_eq!(driver_gross(1, 499999).unwrap(), 0);
        assert_eq!(driver_gross(1, 500000).unwrap(), 1);
        assert_eq!(driver_gross(1, 500001).unwrap(), 1);
        assert_eq!(driver_gross(i64::MAX, 1_000_000).unwrap(), i64::MAX);
        assert_eq!(driver_gross(1000, 0).unwrap(), 0);
        assert!(driver_gross(1000, -1).is_err());
        assert!(driver_gross(1000, 1_000_001).is_err());
        let totals = settlement(
            &[10000, 20000],
            850000,
            &[(Direction::Add, 500), (Direction::Deduct, 1000)],
        )
        .unwrap();
        assert_eq!(
            totals,
            SettlementTotals {
                gross_route_cents: 30000,
                driver_gross_cents: 25500,
                final_payout_cents: 25000
            }
        );
        assert_eq!(
            driver_payroll(totals.final_payout_cents, 125).unwrap(),
            25125
        );
        assert_eq!(
            apply_adjustments(0, &[(Direction::Deduct, 100)]).unwrap(),
            -100
        );
        assert!(apply_adjustments(100, &[(Direction::Add, -1)]).is_err());
        assert!(driver_payroll(100, -1).is_err());
        assert_eq!(settlement(&[], 0, &[]).unwrap().final_payout_cents, 0);
    }
    #[test]
    fn user_period_dates() {
        assert!(validate_period("2026-10-01", "2026-10-19").is_ok());
        assert!(validate_period("2024-02-29", "2024-02-29").is_ok());
        for (a, b) in [
            ("2026-02-29", "2026-03-01"),
            ("2026-10-03", "2026-10-02"),
            ("2026-2-01", "2026-03-01"),
            ("0000-01-01", "2026-01-01"),
            ("2026-04-31", "2026-05-01"),
        ] {
            assert!(validate_period(a, b).is_err());
        }
    }
}
