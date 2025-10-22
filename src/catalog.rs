use crate::value::*;
use pracstro::{coord, time};

#[derive(Clone, Debug, PartialEq)]
pub struct Star {
    pub loc_j2k: coord::Coord,
    pub mag: f64,
    pub pi: time::Angle,
    pub pm_ra: time::Angle,
    pub pm_dec: time::Angle,
}

/// Creates the catalog as a hash table
///
/// This operation takes about 500 µs on my machine
pub fn read() -> std::collections::HashMap<&'static str, CelObj> {
    use pracstro::sol;

    let mut cat = std::collections::HashMap::from([
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
    ]);

    include_str!("dat/stars.csv")
        .lines()
        .skip(1)
        .map(|star| {
            let p: Vec<&str> = star.split(',').collect();
            (
                p[0],
                CelObj::Star(Star {
                    loc_j2k: coord::Coord::from_equatorial(
                        time::Angle::from_degrees(p[1].parse().unwrap()),
                        time::Angle::from_degrees(p[2].parse().unwrap()),
                    ),
                    mag: p[3].parse().unwrap(),
                    pi: time::Angle::from_degrees(p[4].parse::<f64>().unwrap() / 3_600_000.0),
                    pm_ra: time::Angle::from_degrees(p[5].parse::<f64>().unwrap() / 3_600_000.0),
                    pm_dec: time::Angle::from_degrees(p[6].parse::<f64>().unwrap() / 3_600_000.0),
                }),
            )
        })
        .for_each(|(n, s)| {
            cat.insert(n, s);
        });

    cat
}
