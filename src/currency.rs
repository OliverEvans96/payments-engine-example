// Only need 4 decimals precision - f64 would be overkill
pub type CurrencyFloat = f32;

pub fn round_currency(amount: CurrencyFloat) -> CurrencyFloat {
    const NUM_DIGITS: u8 = 4;
    // Round to NUM_DIGITS decimal places
    let multiplier: CurrencyFloat = (10 ^ NUM_DIGITS).into();
    (amount * multiplier).round() / multiplier
}

pub fn floor_currency(amount: CurrencyFloat) -> CurrencyFloat {
    const NUM_DIGITS: u8 = 4;
    // Round to NUM_DIGITS decimal places
    let multiplier: CurrencyFloat = (10 ^ NUM_DIGITS).into();
    (amount * multiplier).floor() / multiplier
}
