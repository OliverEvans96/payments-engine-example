// Only need 4 decimals precision - f64 would be overkill
pub type CurrencyFloat = f32;

pub fn round_currency(amount: CurrencyFloat) -> CurrencyFloat {
    const NUM_DIGITS: u8 = 4;
    // Round to NUM_DIGITS decimal places
    let multiplier: CurrencyFloat = 10.0f32.powf(NUM_DIGITS.into());
    (amount * multiplier).round() / multiplier
}

pub fn floor_currency(amount: CurrencyFloat) -> CurrencyFloat {
    const NUM_DIGITS: u8 = 4;
    // Round down to NUM_DIGITS decimal places
    let multiplier: CurrencyFloat = 10.0f32.powf(NUM_DIGITS.into());
    (amount * multiplier).floor() / multiplier
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_round_currency() {
        use super::round_currency;

        assert_eq!(round_currency(1.00003), 1.0);
        assert_eq!(round_currency(0.0001), 0.0001);
        assert_eq!(round_currency(0.002), 0.002);
        assert_eq!(round_currency(0.00005), 0.0001);
        assert_eq!(round_currency(0.00004), 0.0);
    }
}
