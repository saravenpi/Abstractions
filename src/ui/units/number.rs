//! Numbers as people read them: three significant digits at most, a space between thousands,
//! never scientific notation and no trailing zeros.

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
    let text = format!("{value:.decimals$}");
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
