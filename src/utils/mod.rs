/// Utilities module for common functions

// Add imports at the top
use chrono::{DateTime, TimeZone, Utc};
// Fix the time import - use the full path to avoid confusion with tokio::time
use time::OffsetDateTime;

/// Convert OffsetDateTime to chrono's DateTime<Utc>
pub fn offset_to_chrono(dt: OffsetDateTime) -> DateTime<Utc> {
    let unix_timestamp = dt.unix_timestamp();
    let nanos = dt.nanosecond();
    Utc.timestamp_opt(unix_timestamp, nanos).unwrap()
}

/// Format currency to Indonesian Rupiah format
pub fn format_rupiah(amount: f64) -> String {
    let mut s = String::new();
    let amount_str = format!("{:.0}", amount);
    let chars: Vec<char> = amount_str.chars().collect();
    let len = chars.len();
    
    for (i, c) in chars.iter().enumerate() {
        s.push(*c);
        if (len - i - 1) % 3 == 0 && i < len - 1 {
            s.push('.');
        }
    }
    
    format!("Rp {}", s)
}

/// Validate that a price is not negative
pub fn validate_price(price: f64) -> bool {
    price >= 0.0
}

/// Truncate a string to a maximum length and add ellipsis if truncated
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_rupiah() {
        assert_eq!(format_rupiah(1000000.0), "Rp 1.000.000");
        assert_eq!(format_rupiah(1500.0), "Rp 1.500");
        assert_eq!(format_rupiah(0.0), "Rp 0");
    }
    
    #[test]
    fn test_validate_price() {
        assert!(validate_price(100.0));
        assert!(validate_price(0.0));
        assert!(!validate_price(-10.0));
    }
    
    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("Hello", 10), "Hello");
        assert_eq!(truncate_string("Hello World", 5), "Hello...");
    }
}