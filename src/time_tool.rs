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

pub fn epoch_to_iso8601(mut epoch_secs: i64) -> String {
    // If user accidentally passes milliseconds (e.g. 13 digits)
    if epoch_secs > 999_999_999_999 {
        epoch_secs /= 1000;
    }

    let days = epoch_secs.div_euclid(86400);
    let secs_of_day = epoch_secs.rem_euclid(86400);

    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

pub fn iso8601_to_epoch(iso: &str) -> Result<i64, String> {
    let cleaned = iso.trim().trim_end_matches('Z');
    let parts: Vec<&str> = cleaned.split('T').collect();
    if parts.len() != 2 {
        return Err("Expected ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ".to_string());
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    if date_parts.len() != 3 {
        return Err("Invalid date component in ISO string".to_string());
    }

    let year: i64 = date_parts[0]
        .parse()
        .map_err(|_| "Invalid year in ISO string".to_string())?;
    let month: u32 = date_parts[1]
        .parse()
        .map_err(|_| "Invalid month in ISO string".to_string())?;
    let day: u32 = date_parts[2]
        .parse()
        .map_err(|_| "Invalid day in ISO string".to_string())?;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err("Month or day value is out of valid calendar range".to_string());
    }

    let time_parts: Vec<&str> = parts[1].split(':').collect();
    if time_parts.len() < 2 || time_parts.len() > 3 {
        return Err("Invalid time component in ISO string".to_string());
    }

    let hour: i64 = time_parts[0]
        .parse()
        .map_err(|_| "Invalid hour in ISO string".to_string())?;
    let minute: i64 = time_parts[1]
        .parse()
        .map_err(|_| "Invalid minute in ISO string".to_string())?;
    let second: i64 = if time_parts.len() == 3 {
        time_parts[2]
            .split('.')
            .next()
            .unwrap_or("0")
            .parse()
            .map_err(|_| "Invalid second in ISO string".to_string())?
    } else {
        0
    };

    if hour > 23 || minute > 59 || second > 60 {
        return Err("Time values are out of valid range".to_string());
    }

    let days = days_from_civil(year, month, day);
    let epoch = days * 86400 + hour * 3600 + minute * 60 + second;
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
    }

    #[test]
    fn test_iso8601_to_epoch() {
        let epoch = iso8601_to_epoch("1970-01-01T00:00:00Z").expect("Epoch 0 parse");
        assert_eq!(epoch, 0);

        let epoch2 = iso8601_to_epoch("2023-11-14T22:13:20Z").expect("Timestamp parse");
        assert_eq!(epoch2, 1700000000);
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
    }
}
