use std::collections::HashSet;

use chrono::{DateTime, Utc};

/// Removes duplicate elements from a vector while preserving the original order.
///
/// This function iterates through the input vector, adding each element to a `HashSet`
/// to track uniqueness. If an element has not been seen before, it is included in the
/// resulting vector.
///
/// # Type Parameters
///
/// * `T` - The type of elements in the vector. Must implement `Eq`, `std::hash::Hash`, and `Clone`.
///
/// # Arguments
///
/// * `v` - The input `Vec<T>` from which to remove duplicates.
///
/// # Returns
///
/// A new `Vec<T>` containing only the unique elements from the input vector, in their original order of appearance.
pub(crate) fn dedup_ordered<T>(v: Vec<T>) -> Vec<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    let mut seen = HashSet::new(); // Create an empty HashSet to track seen elements
    v.into_iter()
        .filter(|item| seen.insert(item.clone())) // Filter out elements that have already been seen
        .collect() // Collect the unique elements into a new vector
}

/// Parses a vector of optional parameters into a `Vec` of `(String, String)` tuples.
///
/// This utility function is used to construct query parameters for HTTP requests.
/// It filters out any `None` values and converts `Some` values to their string representation.
///
/// # Type Parameters
///
/// * `T` - The type of the parameter values, which must implement `ToString`.
///
/// # Arguments
///
/// * `params` - A `Vec` of `(&str, Option<T>)` tuples, where the first element is the parameter key
///              and the second is an `Option` containing the parameter value.
///
/// # Returns
///
/// A `Vec` of `(String, String)` tuples, representing the key-value pairs of the present parameters.
pub(crate) fn parse_params<T: ToString>(params: Vec<(&str, Option<T>)>) -> Vec<(String, String)> {
    params
        .into_iter()
        .filter_map(|(key, value)| value.map(|v| (key.to_string(), v.to_string())))
        .collect()
}

/// Converts a `DateTime<Utc>` or `String` to an epoch timestamp in milliseconds.
pub(crate) fn time_to_epoch_ms(date: Option<DateTime<Utc>>) -> Option<String> {
    date.map(|d| d.timestamp_millis().to_string())
}

/// Converts a `DateTime<Utc>` to a "YYYY-MM-DD" string.
pub(crate) fn time_to_yyyymmdd(date: Option<DateTime<Utc>>) -> Option<String> {
    date.map(|d| d.format("%Y-%m-%d").to_string())
}

/// Formats an option contract into the Schwab-standard symbol format.
/// e.g., format_option_symbol("AAPL", "250919", 'C', 232.5) -> "AAPL  250919C00232500"
#[allow(dead_code)]
pub fn format_option_symbol(
    ticker: &str,
    yymmdd: &str,
    side: char,
    strike: f64,
) -> String {
    // 1. Pad the ticker to 6 characters
    let padded_ticker = format!("{:<6}", ticker);

    // 2. Format the strike price to 8 digits (5 whole, 3 decimal)
    let strike_as_int = (strike * 1000.0).round() as u32;
    let formatted_strike = format!("{:08}", strike_as_int);

    // 3. Combine all parts
    format!(
        "{}{}{}{}",
        padded_ticker, yymmdd, side, formatted_strike
    )
}
