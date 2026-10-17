pub(crate) fn parse_rfc3339_ms(text: &str) -> Option<u64> {
    let text = text.trim();
    let (date, rest) = text.split_once('T')?;
    let mut date_parts = date.split('-');
    let year: i64 = date_parts.next()?.parse().ok()?;
    let month: i64 = date_parts.next()?.parse().ok()?;
    let day: i64 = date_parts.next()?.parse().ok()?;

    let time = rest
        .trim_end_matches('Z')
        .split_once('+')
        .map(|(t, _)| t)
        .unwrap_or_else(|| rest.trim_end_matches('Z'));
    let mut time_parts = time.split(':');
    let hour: i64 = time_parts.next()?.parse().ok()?;
    let minute: i64 = time_parts.next()?.parse().ok()?;
    let seconds_field = time_parts.next()?;
    let (secs, frac) = seconds_field
        .split_once('.')
        .unwrap_or((seconds_field, "0"));
    let second: i64 = secs.parse().ok()?;
    let millis: i64 = format!("{frac:0<3}")[..3].parse().ok()?;

    // Days from the civil calendar (Howard Hinnant's algorithm), which is
    // exact and needs no table.
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;

    let total = ((days * 86_400 + hour * 3_600 + minute * 60 + second) * 1_000) + millis;
    u64::try_from(total).ok()
}

#[cfg(test)]
mod audit_tests {
    use super::*;
    #[test]
    fn original_utc_control() {
        assert_eq!(parse_rfc3339_ms("2026-08-10T14:31:05.1234567Z"), Some(1_786_372_265_123));
        assert_eq!(parse_rfc3339_ms("not a time"), None);
    }
    #[test]
    fn positive_offset_is_silently_ignored() {
        let result = parse_rfc3339_ms("2026-08-10T14:31:05.123+05:30");
        assert_eq!(result, Some(1_786_372_265_123));
        assert_ne!(result, Some(1_786_352_465_123));
        println!("positive-offset found={result:?}, required=Some(1786352465123)");
    }
    #[test]
    fn negative_offset_is_ignored_or_unknown_depending_on_fraction() {
        assert_eq!(parse_rfc3339_ms("2026-08-10T14:31:05.123-05:30"), Some(1_786_372_265_123));
        assert_eq!(parse_rfc3339_ms("2026-08-10T14:31:05-05:30"), None);
    }
    #[test]
    fn malformed_unicode_fraction_panics() {
        assert!(std::panic::catch_unwind(|| parse_rfc3339_ms("2026-08-10T14:31:05.ééZ")).is_err());
    }
    #[test]
    fn overflowing_year_panics_in_checked_native_build() {
        assert!(std::panic::catch_unwind(|| parse_rfc3339_ms("9223372036854775807-08-10T14:31:05Z")).is_err());
    }
    #[test]
    fn invalid_february_date_is_accepted() {
        assert!(parse_rfc3339_ms("2026-02-30T14:31:05Z").is_some());
    }
}
