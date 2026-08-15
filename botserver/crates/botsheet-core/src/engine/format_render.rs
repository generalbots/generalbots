//! Scientific and fraction number rendering (#785, E7).
//!
//! The `0.00E+00` and `# ?/?` style numeric renderers. Kept separate from the
//! format parser so `formats.rs` stays under the file-size ceiling.

/// Renders a number in `0.00E+00` style: a mantissa with the requested decimal
/// places and a two-digit, sign-prefixed exponent.
pub(crate) fn render_scientific(n: f64, decimal_digits: usize) -> String {
    if n == 0.0 {
        return format!("0.{}E+00", "0".repeat(decimal_digits));
    }
    let exp = n.abs().log10().floor() as i32;
    let mantissa = n / 10f64.powi(exp);
    let factor = 10f64.powi(decimal_digits as i32);
    let rounded = (mantissa * factor).round() / factor;
    format!("{:.*}E{:+03}", decimal_digits, rounded, exp)
}

/// Renders a number as the closest fraction with a denominator up to
/// `10^denom_digits - 1` (Excel's `# ?/?` and `# ??/??` behaviour).
pub(crate) fn render_fraction(n: f64, denom_digits: usize) -> String {
    let max_denom = 10usize.pow(denom_digits as u32) - 1;
    let whole = n.trunc();
    let frac_abs = (n - whole).abs();

    let mut best_num = 0usize;
    let mut best_den = 1usize;
    let mut best_err = f64::MAX;
    for den in 1..=max_denom.max(1) {
        let num = (frac_abs * den as f64).round() as usize;
        let err = ((num as f64 / den as f64) - frac_abs).abs();
        if err < best_err {
            best_err = err;
            best_num = num;
            best_den = den;
        }
    }

    let sign = if n < 0.0 { "-" } else { "" };
    let whole_part = if whole != 0.0 {
        format!("{} ", whole.abs() as i64)
    } else {
        String::new()
    };
    if best_num == 0 {
        format!("{sign}{}", whole.abs() as i64)
    } else {
        format!("{sign}{whole_part}{best_num}/{best_den}")
    }
}
