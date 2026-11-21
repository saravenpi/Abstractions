//! Numbers as people read them: three significant digits at most, a space between thousands,
//! never scientific notation and no trailing zeros. [`fixed`] is the one place a number is
//! rounded to decimals, so no quantity is ever written as a negative zero.

/// Returns `value` with exactly `decimals` decimals. A value that rounds to zero is written
/// without a sign: "0.0", never "-0.0", whether it was a negative zero or a small negative
/// number.
pub fn fixed(value: impl Into<f64>, decimals: usize) -> String {
    let value = value.into();
    let text = format!("{value:.decimals$}");
    match text.strip_prefix('-') {
        Some(rest) if rest.bytes().all(|b| b == b'0' || b == b'.') => rest.to_string(),
        _ => text,
    }
}

/// Returns `value` with three significant digits, a space between groups of thousands and no
/// zeros at the end of its decimals.
pub fn plain(value: f64) -> String {
    let value = value.max(0.0);
    if value >= 100.0 {
        let step = 10f64.powi(value.log10().floor() as i32 - 2);
        return spaced(((value / step).round() * step) as u64);
    }
    let decimals = match value {
        v if v < 10.0 => 2,
        _ => 1,
    };
    let text = fixed(value, decimals);
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Returns `value` in whole units with a space between groups of three digits.
pub fn spaced(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();
    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            grouped.push(' ');
        }
        grouped.push(digit);
    }
    grouped
}
