use pracstro::time;
use value::*;

pub mod output;
pub mod parse;
pub mod query;
pub mod value;

/// Handles the reading and querying of the catalog of celestial objects
mod catalog {
    use crate::value::*;

    /// Creates the catalog as a hash table
    pub fn read() -> std::collections::HashMap<&'static str, CelObj> {
        use pracstro::sol;
        std::collections::HashMap::from([
            ("sun", CelObj::Sun),
            ("mercury", CelObj::Planet(sol::MERCURY)),
            ("venus", CelObj::Planet(sol::VENUS)),
            ("moon", CelObj::Moon),
            ("mars", CelObj::Planet(sol::MARS)),
            ("jupiter", CelObj::Planet(sol::JUPITER)),
            ("saturn", CelObj::Planet(sol::SATURN)),
            ("uranus", CelObj::Planet(sol::URANUS)),
            ("neptune", CelObj::Planet(sol::NEPTUNE)),
            ("pluto", CelObj::Planet(sol::PLUTO)),
        ])
    }

    /// Gets an object from a catalog
    pub fn get(s: &str, catalog: &std::collections::HashMap<&str, CelObj>) -> Option<CelObj> {
        Some(catalog.get(s)?.clone())
    }
}

/// pracstro provides a way to do this, but that isn't functional in a lot of contexts
///
/// Used in ephemeris generation and date reading
mod timestep {
    use chrono::prelude::*;
    use pracstro::time;
    #[derive(Copy, Clone, Debug, PartialEq)]
    /// Most things can be represented as seconds or months
    /// * 1 second: 1 second
    /// * 1 minute: 60 seconds
    /// * 1 hour: 3600 seconds
    /// * 1 day: 86400 seconds
    /// * 1 week: 604800 seconds
    /// * 1 month: 1 month
    /// * 1 year: 12 months
    pub enum Step {
        S(f64),
        M(chrono::Months),
    }
    pub fn step_forward_date(d: time::Date, s: Step) -> time::Date {
        match s {
            Step::S(sec) => time::Date::from_julian(d.julian() + (sec.abs() / 86400.0)),
            Step::M(m) => time::Date::from_unix(
                (DateTime::from_timestamp(d.unix() as i64, 0).unwrap() + m).timestamp() as f64,
            ),
        }
    }
    pub fn step_back_date(d: time::Date, s: Step) -> time::Date {
        match s {
            Step::S(sec) => time::Date::from_julian(d.julian() - (sec.abs() / 86400.0)),
            Step::M(m) => time::Date::from_unix(
                (DateTime::from_timestamp(d.unix() as i64, 0).unwrap() - m).timestamp() as f64,
            ),
        }
    }
}

fn main() {
    use clap::{arg, command};
    let matches = command!()
    	.help_template("{before-help}{name} ({version}) - {about-with-newline}\n{usage-heading} {usage}\n\n{all-args}{after-help}\n\nWritten by {author}")
        .arg(
            arg!(-d --date [Date] "Set the date")
                .value_parser(parse::date)
                .default_value("now"),
        )
        .arg(
            arg!(-l --latlong ["Latitude,Longitude"] "Set the latitude/longitude")
                .value_parser(parse::latlong)
                .default_value("none"),
        )
        .arg(arg!(-E --ephem ["Start,Step,End"] "Generates Table").value_parser(parse::ephemq))
        .arg(
            arg!(-T --format [Format] "Output Format")
                .value_parser(["term", "csv", "json"])
                .default_value("term"),
        )
        .arg(arg!([object] "Celestial Object").required(true).value_parser(parse::object))
        .arg(arg!([properties] ... "Properties").required(true).value_parser(parse::property))
        .get_matches();

    let mut myrf: RefFrame = RefFrame {
        latlong: *matches.get_one("latlong").unwrap(),
        date: *matches.get_one("date").unwrap(),
    };
    let formatter = match matches.get_one::<String>("format").unwrap().as_str() {
        "term" => output::TERM,
        "csv" => output::CSV,
        "json" => output::JSON,
        _ => todo!(),
    };

    let obj = matches.get_one::<CelObj>("object").unwrap();
    let propl: Vec<query::Property> = matches
        .get_many::<query::Property>("properties")
        .unwrap()
        .cloned()
        .collect();

    let q = |myrf: RefFrame| {
        query::run(obj, &propl, &myrf).unwrap_or_else(|x| panic!("Failed to parse query: {x}"))
    };

    (formatter.start)();

    if let Some((start, step, end)) =
        matches.get_one::<(time::Date, timestep::Step, time::Date)>("ephem")
    {
        myrf.date = *start;
        (formatter.propheader)(&propl);
        while myrf.date.julian() < end.julian() {
            (formatter.ephemq)(q(myrf), &propl, myrf.date);
            myrf.date = timestep::step_forward_date(myrf.date, *step);
        }
    } else {
        (formatter.query)(q(myrf));
    }

    (formatter.footer)();
}
