use thiserror::Error;

#[derive(Error, Debug)]
pub enum MHError {
    #[error("Invalid grid format `{0}`")]
    InvalidGrid(String),
    #[error("Invalid grid length {0}, only 4/6/8/10 supported")]
    InvalidGridLength(usize),
    #[error("Invalid Longitude/Latitude: `{0}`/`{1}`")]
    InvalidLongLat(f64, f64),
    #[error("unknown error when generating grid string")]
    Unknown,
}

// Grid squares are string representations of the latitude and longitude. A good introduction to how to calculate them is in:
// http://www.w8bh.net/grid_squares.pdf
//
// The format is: FFSSssEEee
// Field / Square / Subsquare / Extended Square / Superextended Square
// Each covering for long/lat:
// Field: 20 / 10 degrees for long / lat
// Square: 2 / 1 degrees
// Subsquare: 5 / 2.5 minutes
// Extended Square: 30 / 15 seconds
// Superextended Square: 1.25 / 0.625 seconds
// Note that the enumeration begins at south pole (so 90 degrees off on latitude) and is
// unsigned positive, so needs to be subtracted by 180 to get +/- longitude.

static LONG_OFFSET: f64 = 180.0;
static LAT_OFFSET: f64 = 90.0;

static LONG_F: f64 = 20.0;
static LAT_F: f64 = 10.0;
static LONG_SQ: f64 = 2.0;
static LAT_SQ: f64 = 1.0;
static LONG_SSQ: f64 = 5.0 / 60.0;
static LAT_SSQ: f64 = 2.5 / 60.0;
static LONG_ESQ: f64 = 30.0 / 60.0 / 60.0;
static LAT_ESQ: f64 = 15.0 / 60.0 / 60.0;
static LONG_SESQ: f64 = 1.25 / 60.0 / 60.0;
static LAT_SESQ: f64 = 0.625 / 60.0 / 60.0;

static LONG_MULT: [f64; 5] = [LONG_F, LONG_SQ, LONG_SSQ, LONG_ESQ, LONG_SESQ];
static LAT_MULT: [f64; 5] = [LAT_F, LAT_SQ, LAT_SSQ, LAT_ESQ, LAT_SESQ];

pub fn grid_to_longlat(grid: &str) -> Result<(f64, f64), MHError> {
    // Validate alpha/digit format
    // FIXME: Actual values should be A-R 0-9 a-x 0-9 A-X
    let d = |a: char| a.is_ascii_digit();
    let l = |a: char| a.is_ascii_alphabetic();
    let checks = [l, l, d, d, l, l, d, d, l, l];
    let check = grid
        .chars()
        .zip(checks)
        .map(|(c, lmb)| lmb(c))
        .collect::<Vec<bool>>();

    // If any of them are false, we've got an invalid grid string
    if check.iter().filter(|b| !*b).count() != 0 {
        return Err(MHError::InvalidGrid(grid.to_string()));
    }

    // Also make sure the length is even (and not 2)
    match grid.len() {
        4 | 6 | 8 | 10 => {}
        l => return Err(MHError::InvalidGridLength(l)),
    }

    // Now it's just a matter of calculating the offsets from the grid
    let vals: Vec<u32> = "AA00AA00AA"
        .chars()
        .zip(grid.chars())
        .map(|(t, c)| (c.to_ascii_uppercase() as u32) - (t as u32))
        .collect();

    // And multiplying each of them with their per-unit value
    let long: f64 = vals
        .iter()
        .step_by(2)
        .zip(LONG_MULT)
        .map(|(&v, m)| v as f64 * m)
        .sum();
    let lat: f64 = vals
        .iter()
        .skip(1)
        .step_by(2)
        .zip(LAT_MULT)
        .map(|(&v, m)| v as f64 * m)
        .sum();

    // Move the returned value into the middle of the precision given.
    // This avoids imprecision due to rounding if doing grid->longlat->grid
    // (We do this in a unit testcase)
    let idx = grid.len() / 2 - 1;
    let long = long + LONG_MULT[idx] / 2.0;
    let lat = lat + LAT_MULT[idx] / 2.0;

    // Finally, adjust for origin offsets
    Ok((long - LONG_OFFSET, lat - LAT_OFFSET))
}

pub fn longlat_to_grid(long: f64, lat: f64, precision: usize) -> Result<String, MHError> {
    let charoff = |base: char, off: u32| std::char::from_u32(base as u32 + off);

    // It only makes sense to have 4+ even number of characters in a grid square
    match precision {
        4 | 6 | 8 | 10 => {},
        p => return Err(MHError::InvalidGridLength(p)),
    }

    if long > 180.0 || long < -180.0 || lat < -180.0 || lat > 180.0 {
        return Err(MHError::InvalidLongLat(long, lat));
    }

    // Do the math to calculate each position, per the w8bh website
    let long = long + LONG_OFFSET;
    let lat = lat + LAT_OFFSET;
    let mut vals = Vec::new();
    vals.push(long / LONG_F);
    vals.push(lat / LAT_F);
    vals.push(long % LONG_F / LONG_SQ);
    vals.push(lat % LAT_F / LAT_SQ);
    vals.push(long % LONG_SQ / LONG_SSQ);
    vals.push(lat % LAT_SQ / LAT_SSQ);
    vals.push(long % LONG_SSQ / LONG_ESQ);
    vals.push(lat % LAT_SSQ / LAT_ESQ);
    vals.push(long % LONG_ESQ / LONG_SESQ);
    vals.push(lat % LAT_ESQ / LAT_SESQ);

    vals.truncate(precision);

    let grid: Option<String> = "AA00aa00AA"
        .chars()
        .zip(vals)
        .map(|(b, o)| charoff(b, o as u32))
        .collect();
    match grid {
        Some(g) => Ok(g),
        None => Err(MHError::Unknown),
    }
}

// Calculate the distance between two grids, using the haversine
// formula:
// a = sin²(Δφ/2) + cos φ1 ⋅ cos φ2 ⋅ sin²(Δλ/2)
// c = 2 ⋅ atan2( √a, √(1−a) )
// d = R ⋅ c
// where:
//  φ is latitude, λ is longitude, R is earth’s radius (mean radius = 6,371km);
//  Bearing:
//  θ = atan2( sin Δλ ⋅ cos φ2 , cos φ1 ⋅ sin φ2 − sin φ1 ⋅ cos φ2 ⋅ cos Δλ )

pub fn grid_dist_bearing(from: &str, to: &str) -> Result<(f64, f64), MHError> {
    static RADIUS: f64 = 6371.0;
    let (from_long, from_lat) = grid_to_longlat(from)?;
    let (to_long, to_lat) = grid_to_longlat(to)?;

    #[allow(non_snake_case)]
    let Δλ = (to_long - from_long).to_radians();
    #[allow(non_snake_case)]
    let Δφ = (to_lat - from_lat).to_radians();
    let φ1 = from_lat.to_radians();
    let φ2 = to_lat.to_radians();

    let a: f64 = (Δφ / 2.0).sin().powi(2) + φ1.cos() * φ2.cos() * (Δλ / 2.0).sin().powi(2);
    let c: f64 = 2.0 * (a.sqrt()).atan2((1.0 - a).sqrt());

    let dist = RADIUS * c;
    let bearing = (Δλ.sin() * φ2.cos()).atan2(φ1.cos() * φ2.sin() - φ1.sin() * φ2.cos() * Δλ.cos());
    let bearing = (bearing.to_degrees() + 360.0) % 360.0;

    Ok((dist, bearing))
}

pub fn grid_distance(from: &str, to: &str) -> Result<f64, MHError> {
    let (dist, _) = grid_dist_bearing(from, to)?;
    Ok(dist)
}

pub fn grid_bearing(from: &str, to: &str) -> Result<f64, MHError> {
    let (_, bearing) = grid_dist_bearing(from, to)?;
    Ok(bearing)
}

#[cfg(test)]
mod tests {
    use super::*;

    // From https://stackoverflow.com/questions/30856285/assert-eq-with-floating-point-numbers-and-delta
    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            let x = $x as f64;
            let y = $y as f64;
            if !((x - y).abs() < $d || (y - x).abs() < $d) {
                panic!();
            }
        };
    }

    // These values come out of the PDF referenced at the top of this file
    static TEST_GRID: &str = &"FM18lv53SL";
    static TEST_LONG: f64 = -77.035278;
    static TEST_LAT: f64 = 38.889484;

    fn precision_n(n: usize) {
        let grid = longlat_to_grid(TEST_LONG, TEST_LAT, n).unwrap();
        let mut check = String::from(TEST_GRID);
        check.truncate(n);
        println!("Grid ({}): {}", n, check);
        assert_eq!(grid, check);
    }

    #[test]
    fn precision_10() {
        precision_n(10);
    }

    #[test]
    fn precision_8() {
        precision_n(8);
    }

    #[test]
    fn precision_6() {
        precision_n(6);
    }

    #[test]
    fn precision_4() {
        precision_n(4);
    }

    #[test]
    fn precision_inval() {
        let grid = longlat_to_grid(TEST_LONG, TEST_LAT, 5);
        assert!(grid.is_err());
    }

    #[test]
    fn precision_inval_lat() {
        let grid = longlat_to_grid(TEST_LONG, 921.0, 10);
        assert!(grid.is_err());
    }

    #[test]
    fn precision_inval_long() {
        let grid = longlat_to_grid(-201.0, TEST_LAT, 10);
        assert!(grid.is_err());
    }

    fn longlat_n(n: usize) {
        let mut grid_in = String::from(TEST_GRID);
        grid_in.truncate(n);

        let ll = grid_to_longlat(&grid_in.as_str());
        assert!(!ll.is_err());

        // Make sure it's within the margin of error of the smallest field
        let (long, lat) = ll.unwrap();
        assert_delta!(long, TEST_LONG, LONG_MULT[n/2-1]);
        assert_delta!(lat, TEST_LAT, LAT_MULT[n/2-1]);

        // Let's convert it back to grid and compare
        let grid = longlat_to_grid(long, lat, n).unwrap();
        assert_eq!(grid_in, grid);
    }

    #[test]
    fn longlat10() {
        longlat_n(10);
    }

    #[test]
    fn longlat8() {
        longlat_n(8);
    }

    #[test]
    fn longlat6() {
        longlat_n(6);
    }

    #[test]
    fn longlat4() {
        longlat_n(4);
    }

    #[test]
    fn longlat_invalid() {
        let ret = grid_to_longlat("AI021");
        assert!(ret.is_err());
        let ret = grid_to_longlat("AIA2");
        assert!(ret.is_err());
        let ret = grid_to_longlat("🤷I00");
        assert!(ret.is_err());
        let ret = grid_to_longlat("AA00AA00AA00");
        assert!(ret.is_err());
        let ret = grid_to_longlat("AA00AA00AA");
        assert!(!ret.is_err());
    }

    #[test]
    fn test_distance_null() {
        let dist = grid_distance(TEST_GRID, TEST_GRID).unwrap();
        assert_eq!(dist, 0.0);
    }

    #[test]
    fn test_distance_home() {
        let dist = grid_distance("CM87um", "KP04ow").unwrap();
        let bear = grid_bearing("CM87um", "KP04ow").unwrap();
        println!("Distance: {} Bearing: {}", dist, bear);
        println!("from: {:?} To: {:?}", grid_to_longlat("CM87um"), grid_to_longlat("KP04ow"));
        assert_delta!(dist, 8189.0, 1.0);
        assert_delta!(bear, 15.224, 0.001);
    }
}
