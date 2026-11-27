use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn current_epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

pub fn epoch_to_iso8601_with_meta(mut epoch: i64) -> (String, bool) {
    // Auto-detect milliseconds for 13+ digit positive or negative timestamps
    let detected_millis = epoch.abs() > 999_999_999_999;
    if detected_millis {
        epoch = epoch.div_euclid(1000);
    }

    let days = epoch.div_euclid(86400);
    let secs_of_day = epoch.rem_euclid(86400);

    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;

    let year_str = if year < 0 {
        format!("-{:04}", year.unsigned_abs())
    } else {
        format!("{year:04}")
    };

    (
        format!("{year_str}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"),
        detected_millis,
    )
}

pub fn epoch_to_iso8601(epoch_secs: i64) -> String {
    epoch_to_iso8601_with_meta(epoch_secs).0
}

fn parse_unsigned_digits<T: std::str::FromStr>(s: &str, label: &str) -> Result<T, String> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("Invalid {label} in ISO string"));
    }
    s.parse::<T>()
        .map_err(|_| format!("Invalid {label} in ISO string"))
}

fn parse_timezone_suffix(time_part: &str) -> Result<(&str, i64), String> {
    if let Some(stripped) = time_part
        .strip_suffix('Z')
        .or_else(|| time_part.strip_suffix('z'))
    {
        return Ok((stripped, 0));
    }

    // Look for '+' or '-' timezone offset after HH:MM (at least index 5)
    if let Some(pos) = time_part.rfind(['+', '-']) {
        if pos >= 4 {
            let sign: i64 = if time_part.as_bytes()[pos] == b'-' {
                -1
            } else {
                1
            };
            let tz = &time_part[pos + 1..];
            let (tz_h, tz_m) = if let Some((h_str, m_str)) = tz.split_once(':') {
                if h_str.len() != 2 || m_str.len() != 2 {
                    return Err(
                        "Invalid timezone offset in ISO string (expected +HH:MM)".to_string()
                    );
                }
                (
                    parse_unsigned_digits::<i64>(h_str, "timezone hour")?,
                    parse_unsigned_digits::<i64>(m_str, "timezone minute")?,
                )
            } else if tz.len() == 4 {
                (
                    parse_unsigned_digits::<i64>(&tz[..2], "timezone hour")?,
                    parse_unsigned_digits::<i64>(&tz[2..], "timezone minute")?,
                )
            } else if tz.len() == 2 {
                (parse_unsigned_digits::<i64>(tz, "timezone hour")?, 0)
            } else {
                return Err("Invalid timezone offset in ISO string".to_string());
            };

            if tz_h > 23 || tz_m > 59 {
                return Err("Timezone offset is out of valid range".to_string());
            }
            return Ok((&time_part[..pos], sign * (tz_h * 3600 + tz_m * 60)));
        }
    }

    Ok((time_part, 0))
}

pub fn iso8601_to_epoch(iso: &str) -> Result<i64, String> {
    let trimmed = iso.trim();
    let (date_str, raw_time_str) = trimmed
        .split_once(['T', 't', ' '])
        .ok_or_else(|| "Expected ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ".to_string())?;

    let date_parts: Vec<&str> = date_str.split('-').collect();
    if date_parts.len() != 3 {
        return Err("Invalid date component in ISO string".to_string());
    }

    let year: i64 = date_parts[0]
        .parse()
        .map_err(|_| "Invalid year in ISO string".to_string())?;
    let month: u32 = parse_unsigned_digits(date_parts[1], "month")?;
    let day: u32 = parse_unsigned_digits(date_parts[2], "day")?;

    let max_day = days_in_month(year, month);
    if !(1..=12).contains(&month) || day < 1 || day > max_day {
        return Err(format!(
            "Calendar date {year:04}-{month:02}-{day:02} is out of valid range"
        ));
    }

    let (time_no_tz, offset_secs) = parse_timezone_suffix(raw_time_str)?;

    let time_parts: Vec<&str> = time_no_tz.split(':').collect();
    if time_parts.len() < 2 || time_parts.len() > 3 {
        return Err("Invalid time component in ISO string".to_string());
    }

    let hour: i64 = parse_unsigned_digits(time_parts[0], "hour")?;
    let minute: i64 = parse_unsigned_digits(time_parts[1], "minute")?;
    let second: i64 = if time_parts.len() == 3 {
        if let Some((sec_part, frac_part)) = time_parts[2].split_once('.') {
            if frac_part.is_empty() || !frac_part.bytes().all(|b| b.is_ascii_digit()) {
                return Err("Invalid fractional seconds in ISO string".to_string());
            }
            parse_unsigned_digits(sec_part, "second")?
        } else {
            parse_unsigned_digits(time_parts[2], "second")?
        }
    } else {
        0
    };

    if hour > 23 || minute > 59 || second > 60 {
        return Err("Time values are out of valid range".to_string());
    }

    let days = days_from_civil(year, month, day);
    let epoch = days * 86400 + hour * 3600 + minute * 60 + second - offset_secs;
    Ok(epoch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_epochs() {
        let secs = current_epoch_secs();
        let millis = current_epoch_millis();
        assert!(secs > 1_700_000_000);
        assert!(millis > 1_700_000_000_000);
    }

    #[test]
    fn test_epoch_to_iso8601() {
        assert_eq!(epoch_to_iso8601(0), "1970-01-01T00:00:00Z");
        assert_eq!(epoch_to_iso8601(1700000000), "2023-11-14T22:13:20Z");
        assert_eq!(epoch_to_iso8601(-86400), "1969-12-31T00:00:00Z");
        assert_eq!(epoch_to_iso8601(1700000000000), "2023-11-14T22:13:20Z");
        assert_eq!(epoch_to_iso8601(-8640000000000), "1696-03-17T00:00:00Z");
    }

    #[test]
    fn test_iso8601_to_epoch() {
        let epoch = iso8601_to_epoch("1970-01-01T00:00:00Z").expect("Epoch 0 parse");
        assert_eq!(epoch, 0);

        let epoch2 = iso8601_to_epoch("2023-11-14T22:13:20Z").expect("Timestamp parse");
        assert_eq!(epoch2, 1700000000);

        let with_offset = iso8601_to_epoch("2026-01-01T05:30:00+05:30").expect("Offset parse");
        let utc = iso8601_to_epoch("2026-01-01T00:00:00Z").expect("UTC parse");
        assert_eq!(with_offset, utc);

        let with_frac = iso8601_to_epoch("2024-02-29T12:00:00.123Z").expect("Leap year + frac");
        assert_eq!(epoch_to_iso8601(with_frac), "2024-02-29T12:00:00Z");
    }

    #[test]
    fn test_time_roundtrip() {
        let target = "2026-11-02T12:30:00Z";
        let epoch = iso8601_to_epoch(target).expect("parse");
        let formatted = epoch_to_iso8601(epoch);
        assert_eq!(formatted, target);
    }

    #[test]
    fn test_invalid_iso_format() {
        assert!(iso8601_to_epoch("not-a-date").is_err());
        assert!(iso8601_to_epoch("2026-99-99T99:99:99Z").is_err());
        assert!(iso8601_to_epoch("2026-02-31T00:00:00Z").is_err());
        assert!(iso8601_to_epoch("2025-02-29T00:00:00Z").is_err());
        assert!(iso8601_to_epoch("2026-04-31T00:00:00Z").is_err());
        assert!(iso8601_to_epoch("2026-01-01T-5:00:00Z").is_err());
        assert!(iso8601_to_epoch("2026-01-01T12:00:00ZZZ").is_err());
        assert!(iso8601_to_epoch("2026-01-01T12:00:00.abcZ").is_err());
    }
}
