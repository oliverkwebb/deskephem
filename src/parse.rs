use crate::query::Property;
use crate::timestep;
use chrono::prelude::*;
use pracstro::time;

fn suffix_num(s: &str, j: &str) -> Option<f64> {
    s.strip_suffix(j)?.parse::<f64>().ok()
}

pub fn property(sm: &str) -> Result<Property, &'static str> {
    let s = &sm.to_lowercase();
    match s.as_str() {
        "equ" | "equa" | "equatorial" => Ok(Property::Equatorial),
        "horiz" | "horizontal" => Ok(Property::Horizontal),
        "ecl" | "ecliptic" => Ok(Property::Ecliptic),
        "dist" | "distance" => Ok(Property::Distance),
        "mag" | "magnitude" | "brightness" => Ok(Property::Magnitude),
        "phase" => Ok(Property::PhaseDefault),
        "phaseemoji" => Ok(Property::PhaseEmoji),
        "phasename" => Ok(Property::PhaseName),
        "angdia" => Ok(Property::AngDia),
        "phaseprecent" | "illumfrac" => Ok(Property::IllumFrac),
        "rise" => Ok(Property::Rise),
        "set" => Ok(Property::Set),
        _ => Err("Unknown Property"),
    }
}

/// A step in time, returns (years, months, days, hours, minutes, seconds)
pub fn step(sm: &str) -> Result<timestep::Step, &'static str> {
    let s = &sm.to_lowercase(); // This can usually be guaranteed, except in argument parsing
    if let Some(n) = suffix_num(s, "y") {
        Ok(timestep::Step::M(chrono::Months::new(n as u32 * 12)))
    } else if let Some(n) = suffix_num(s, "mon") {
        Ok(timestep::Step::M(chrono::Months::new(n as u32)))
    } else if let Some(n) = suffix_num(s, "w") {
        Ok(timestep::Step::S(n * 7.0 * 86400.0))
    } else if let Some(n) = suffix_num(s, "d") {
        Ok(timestep::Step::S(n * 86400.0))
    } else if let Some(n) = suffix_num(s, "h") {
        Ok(timestep::Step::S(n * 3600.0))
    } else if let Some(n) = suffix_num(s, "min") {
        Ok(timestep::Step::S(n * 60.0))
    } else if let Some(n) = suffix_num(s, "s") {
        Ok(timestep::Step::S(n))
    } else {
        Err("Bad interval")
    }
}

pub fn ephemq(s: &str) -> Result<(time::Date, timestep::Step, time::Date), &'static str> {
    let mut eq = s.split(',');
    let start = eq.next().ok_or("Bad CSV")?;
    let ste = eq.next().ok_or("Bad CSV")?;
    let end = eq.next().ok_or("Bad CSV")?;
    Ok((date(start)?, step(ste)?, date(end)?))
}

/// The inbuilt RFC3339/ISO6901 date parser in chrono does not support subsets of the formatting.
pub fn date(sm: &str) -> Result<time::Date, &'static str> {
    let s = &sm.to_lowercase(); // This can usually be guaranteed, except in argument parsing
    if s == "now" {
        Ok(time::Date::now())
    } else if s.starts_with("-") {
        Ok(timestep::step_back_date(
            time::Date::now(),
            step(s.strip_prefix("-").ok_or("Bad prefix")?)?,
        ))
    } else if s.starts_with("+") {
        Ok(timestep::step_forward_date(
            time::Date::now(),
            step(s.strip_prefix("+").ok_or("Bad prefix")?)?,
        ))
    } else if s.starts_with("@") {
        Ok(time::Date::from_unix(
            s.strip_prefix("@")
                .ok_or("")?
                .parse()
                .ok()
                .ok_or("Bad Number")?,
        ))
    } else if let Some(n) = suffix_num(s, "u") {
        Ok(time::Date::from_unix(n))
    } else if let Some(n) = suffix_num(s, "jd") {
        Ok(time::Date::from_julian(n))
    } else if let Some(n) = suffix_num(s, "j") {
        Ok(time::Date::from_julian(n))
    } else if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        Ok(time::Date::from_unix(d.timestamp() as f64))
    } else if let Ok(d) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dt%H:%M:%S") {
        Ok(time::Date::from_unix(d.and_utc().timestamp() as f64))
    } else if let Ok(d) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dt%H:%M") {
        Ok(time::Date::from_unix(d.and_utc().timestamp() as f64))
    } else if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        Ok(time::Date::from_unix(
            NaiveDateTime::from(d).and_utc().timestamp() as f64,
        ))
    } else {
        Err("Invalid Date")
    }
}

pub fn angle(s: &str) -> Result<time::Period, &'static str> {
    let sl = &s.to_lowercase(); // This can usually be guaranteed, except in argument parsing
    if let Some(n) = suffix_num(sl, "e") {
        Ok(time::Period::from_degrees(n))
    } else if let Some(n) = suffix_num(sl, "w") {
        Ok(time::Period::from_degrees(-n))
    } else if let Some(n) = suffix_num(sl, "n") {
        Ok(time::Period::from_degrees(n))
    } else if let Some(n) = suffix_num(sl, "s") {
        Ok(time::Period::from_degrees(-n))
    } else if let Some(n) = suffix_num(sl, "d") {
        Ok(time::Period::from_degrees(n))
    } else if let Some(n) = suffix_num(sl, "deg") {
        Ok(time::Period::from_degrees(n))
    } else if let Some(n) = suffix_num(sl, "°") {
        Ok(time::Period::from_degrees(n))
    } else if let Some(n) = suffix_num(sl, "rad") {
        Ok(time::Period::from_radians(n))
    } else {
        Err("Invalid Angle")
    }
}

pub fn latlong(s: &str) -> Result<Option<(time::Period, time::Period)>, &'static str> {
    fn long(s: &str) -> Result<time::Period, &'static str> {
        if let Ok(n) = s.parse::<f64>() {
            Ok(time::Period::from_degrees(n))
        } else {
            angle(s)
        }
    }
    fn lat(s: &str) -> Result<time::Period, &'static str> {
        let unchecked_l = long(s)?;
        if unchecked_l.to_latitude().degrees() > 90.0 {
            Err("Latitude over 90 degrees")
        } else {
            Ok(unchecked_l)
        }
    }
    if s == "none" {
        return Ok(None);
    };
    let mut eq = s.split(',');
    let lats = eq.next().ok_or("Bad CSV")?;
    let longs = eq.next().ok_or("Bad CSV")?;
    Ok(Some((lat(lats)?, long(longs)?)))
}
