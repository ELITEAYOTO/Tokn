pub fn parse_rfc3339_unix_ms(input: &str) -> Option<u64> {
    let (date, time_and_zone) = input.split_once('T')?;
    let (year, month, day) = parse_date(date)?;
    let (time, offset_seconds) = split_zone(time_and_zone)?;
    let (hour, minute, second, millis) = parse_time(time)?;

    let days = days_from_civil(year, month, day)?;
    let local_seconds = days
        .checked_mul(86_400)?
        .checked_add(i64::from(hour) * 3_600)?
        .checked_add(i64::from(minute) * 60)?
        .checked_add(i64::from(second))?;
    let utc_seconds = local_seconds.checked_sub(offset_seconds)?;
    if utc_seconds < 0 {
        return None;
    }

    u64::try_from(utc_seconds)
        .ok()?
        .checked_mul(1_000)?
        .checked_add(u64::from(millis))
}

fn parse_date(input: &str) -> Option<(i64, u32, u32)> {
    let bytes = input.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year = i64::from(parse_ascii_u32(&bytes[0..4])?);
    let month = parse_ascii_u32(&bytes[5..7])?;
    let day = parse_ascii_u32(&bytes[8..10])?;
    let max_day = days_in_month(year, month)?;
    if day == 0 || day > max_day {
        return None;
    }
    Some((year, month, day))
}

fn parse_time(input: &str) -> Option<(u32, u32, u32, u32)> {
    let (clock, fraction) = input
        .split_once('.')
        .map_or((input, None), |(clock, fraction)| (clock, Some(fraction)));
    let bytes = clock.as_bytes();
    if bytes.len() != 8 || bytes[2] != b':' || bytes[5] != b':' {
        return None;
    }

    let hour = parse_ascii_u32(&bytes[0..2])?;
    let minute = parse_ascii_u32(&bytes[3..5])?;
    let second = parse_ascii_u32(&bytes[6..8])?;
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }

    let millis = fraction.map_or(Some(0), parse_fraction_millis)?;
    Some((hour, minute, second, millis))
}

fn parse_ascii_u32(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    bytes.iter().try_fold(0_u32, |value, byte| {
        value.checked_mul(10)?.checked_add(u32::from(*byte - b'0'))
    })
}

fn parse_fraction_millis(input: &str) -> Option<u32> {
    if input.is_empty() || !input.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let mut millis = 0_u32;
    for (index, byte) in input.bytes().take(3).enumerate() {
        let digit = u32::from(byte - b'0');
        millis += match index {
            0 => digit * 100,
            1 => digit * 10,
            _ => digit,
        };
    }
    Some(millis)
}

fn split_zone(input: &str) -> Option<(&str, i64)> {
    if let Some(time) = input.strip_suffix('Z') {
        return Some((time, 0));
    }

    let offset_index = input.rfind(['+', '-'])?;
    if offset_index < 8 {
        return None;
    }
    let (time, offset) = input.split_at(offset_index);
    let offset_bytes = offset.as_bytes();
    if offset_bytes.len() != 6 || offset_bytes[3] != b':' {
        return None;
    }

    let sign = if offset_bytes[0] == b'+' {
        1_i64
    } else if offset_bytes[0] == b'-' {
        -1_i64
    } else {
        return None;
    };
    let hours = parse_ascii_u32(&offset_bytes[1..3])?;
    let minutes = parse_ascii_u32(&offset_bytes[4..6])?;
    if hours > 23 || minutes > 59 {
        return None;
    }

    let seconds = i64::from(hours) * 3_600 + i64::from(minutes) * 60;
    Some((time, sign * seconds))
}

fn days_in_month(year: i64, month: u32) -> Option<u32> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if is_leap_year(year) => Some(29),
        2 => Some(28),
        _ => None,
    }
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> Option<i64> {
    if year < 0 {
        return None;
    }
    let mut adjusted_year = year;
    if month <= 2 {
        adjusted_year -= 1;
    }
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_epoch_and_offsets() {
        assert_eq!(parse_rfc3339_unix_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_rfc3339_unix_ms("1970-01-01T01:00:00+01:00"), Some(0));
    }

    #[test]
    fn parses_fractional_seconds() {
        assert_eq!(
            parse_rfc3339_unix_ms("1970-01-01T00:00:00.123456Z"),
            Some(123)
        );
        assert_eq!(parse_rfc3339_unix_ms("1970-01-01T00:00:00.1Z"), Some(100));
    }

    #[test]
    fn rejects_invalid_calendar_values() {
        assert_eq!(parse_rfc3339_unix_ms("2026-02-30T00:00:00Z"), None);
        assert_eq!(parse_rfc3339_unix_ms("2026-01-01T25:00:00Z"), None);
    }

    #[test]
    fn rejects_non_ascii_malformed_input_without_panicking() {
        assert_eq!(parse_rfc3339_unix_ms("2026-é1-01T00:00:00Z"), None);
        assert_eq!(parse_rfc3339_unix_ms("2026-01-01Té0:00:00Z"), None);
        assert_eq!(parse_rfc3339_unix_ms("2026-01-01T00:00:00+é1:00"), None);
    }
}
