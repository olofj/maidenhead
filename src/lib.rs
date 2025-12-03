use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum MHError {
    InvalidGrid(String),
    InvalidGridLength(usize),
    InvalidLongLat(f64, f64),
    Unknown,
}

impl fmt::Display for MHError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGrid(grid) => write!(f, "Invalid grid format `{grid}`"),
            Self::InvalidGridLength(len) => write!(f, "Invalid grid length {len}, only 4/6/8/10 supported"),
            Self::InvalidLongLat(long, lat) => write!(f, "Invalid Longitude/Latitude: `{long}`/`{lat}`"),
            Self::Unknown => write!(f, "unknown error when generating grid string"),
        }
    }
}

impl Error for MHError {}

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

const LONG_OFFSET: f64 = 180.0;
const LAT_OFFSET: f64 = 90.0;

const LONG_F: f64 = 20.0;
const LAT_F: f64 = 10.0;
const LONG_SQ: f64 = 2.0;
const LAT_SQ: f64 = 1.0;
const LONG_SSQ: f64 = 5.0 / 60.0;
const LAT_SSQ: f64 = 2.5 / 60.0;
const LONG_ESQ: f64 = 30.0 / 60.0 / 60.0;
const LAT_ESQ: f64 = 15.0 / 60.0 / 60.0;
const LONG_SESQ: f64 = 1.25 / 60.0 / 60.0;
const LAT_SESQ: f64 = 0.625 / 60.0 / 60.0;

const LONG_MULT: [f64; 5] = [LONG_F, LONG_SQ, LONG_SSQ, LONG_ESQ, LONG_SESQ];
const LAT_MULT: [f64; 5] = [LAT_F, LAT_SQ, LAT_SSQ, LAT_ESQ, LAT_SESQ];

/// Converts a Maidenhead grid square string to longitude and latitude coordinates.
///
/// # Arguments
/// * `grid` - A grid square string (4, 6, 8, or 10 characters)
///
/// # Returns
/// A tuple of (longitude, latitude) in decimal degrees
///
/// # Errors
/// Returns `MHError::InvalidGrid` if the grid format is invalid
/// Returns `MHError::InvalidGridLength` if the grid length is not 4, 6, 8, or 10
pub fn grid_to_longlat(grid: &str) -> Result<(f64, f64), MHError> {
    // Validate alpha/digit format
    // FIXME: Actual values should be A-R 0-9 a-x 0-9 A-X
    let is_digit = |c: char| c.is_ascii_digit();
    let is_alpha = |c: char| c.is_ascii_alphabetic();
    let pattern = [
        is_alpha, is_alpha, is_digit, is_digit, is_alpha, is_alpha, is_digit, is_digit, is_alpha,
        is_alpha,
    ];

    let is_valid = grid
        .chars()
        .zip(pattern)
        .take(grid.len())
        .all(|(c, check_fn)| check_fn(c));

    if !is_valid {
        return Err(MHError::InvalidGrid(grid.to_string()));
    }

    // Also make sure the length is even (and not 2)
    match grid.len() {
        4 | 6 | 8 | 10 => {}
        l => return Err(MHError::InvalidGridLength(l)),
    }

    // Calculate the offsets from the grid
    let reference = "AA00AA00AA";
    let vals: Vec<u32> = reference
        .chars()
        .zip(grid.chars())
        .map(|(ref_char, grid_char)| (grid_char.to_ascii_uppercase() as u32) - (ref_char as u32))
        .collect();

    // Calculate longitude and latitude by multiplying with per-unit values
    let long: f64 = vals
        .iter()
        .step_by(2)
        .zip(LONG_MULT)
        .map(|(&v, m)| f64::from(v) * m)
        .sum();
    let lat: f64 = vals
        .iter()
        .skip(1)
        .step_by(2)
        .zip(LAT_MULT)
        .map(|(&v, m)| f64::from(v) * m)
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

/// Converts longitude and latitude coordinates to a Maidenhead grid square string.
///
/// # Arguments
/// * `long` - Longitude in decimal degrees (-180.0 to 180.0)
/// * `lat` - Latitude in decimal degrees (-90.0 to 90.0)
/// * `precision` - Number of characters for the grid (4, 6, 8, or 10)
///
/// # Returns
/// A grid square string of the specified precision
///
/// # Errors
/// Returns `MHError::InvalidLongLat` if coordinates are out of range
/// Returns `MHError::InvalidGridLength` if precision is not 4, 6, 8, or 10
pub fn longlat_to_grid(long: f64, lat: f64, precision: usize) -> Result<String, MHError> {
    let charoff = |base: char, off: u32| std::char::from_u32(base as u32 + off);

    // It only makes sense to have 4+ even number of characters in a grid square
    match precision {
        4 | 6 | 8 | 10 => {}
        p => return Err(MHError::InvalidGridLength(p)),
    }

    if !(-180.0..=180.0).contains(&long) || !(-90.0..=90.0).contains(&lat) {
        return Err(MHError::InvalidLongLat(long, lat));
    }

    // Calculate each position value per the w8bh website
    let adj_long = long + LONG_OFFSET;
    let adj_lat = lat + LAT_OFFSET;

    let mut vals = Vec::with_capacity(precision);
    vals.push(adj_long / LONG_F);
    vals.push(adj_lat / LAT_F);
    vals.push(adj_long % LONG_F / LONG_SQ);
    vals.push(adj_lat % LAT_F / LAT_SQ);
    vals.push(adj_long % LONG_SQ / LONG_SSQ);
    vals.push(adj_lat % LAT_SQ / LAT_SSQ);
    vals.push(adj_long % LONG_SSQ / LONG_ESQ);
    vals.push(adj_lat % LAT_SSQ / LAT_ESQ);
    vals.push(adj_long % LONG_ESQ / LONG_SESQ);
    vals.push(adj_lat % LAT_ESQ / LAT_SESQ);

    vals.truncate(precision);

    let base_chars = "AA00aa00AA";
    let grid: Option<String> = base_chars
        .chars()
        .zip(vals)
        .map(|(base, offset)| charoff(base, offset as u32))
        .collect();

    grid.ok_or(MHError::Unknown)
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

/// Calculates the distance and bearing between two grid squares using the Haversine formula.
///
/// # Arguments
/// * `from` - Source grid square string
/// * `to` - Destination grid square string
///
/// # Returns
/// A tuple of (distance in km, bearing in degrees)
///
/// # Errors
/// Returns `MHError` if either grid square is invalid
pub fn grid_dist_bearing(from: &str, to: &str) -> Result<(f64, f64), MHError> {
    const RADIUS: f64 = 6371.0;
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

/// Calculates the distance between two grid squares in kilometers.
///
/// # Arguments
/// * `from` - Source grid square string
/// * `to` - Destination grid square string
///
/// # Returns
/// Distance in kilometers
///
/// # Errors
/// Returns `MHError` if either grid square is invalid
pub fn grid_distance(from: &str, to: &str) -> Result<f64, MHError> {
    let (dist, _) = grid_dist_bearing(from, to)?;
    Ok(dist)
}

/// Calculates the bearing from one grid square to another in degrees.
///
/// # Arguments
/// * `from` - Source grid square string
/// * `to` - Destination grid square string
///
/// # Returns
/// Bearing in degrees (0-360)
///
/// # Errors
/// Returns `MHError` if either grid square is invalid
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
    static TEST_GRID: &str = "FM18lv53SL";
    static TEST_LONG: f64 = -77.035278;
    static TEST_LAT: f64 = 38.889484;

    fn precision_n(n: usize) {
        let grid = longlat_to_grid(TEST_LONG, TEST_LAT, n).unwrap();
        let mut check = String::from(TEST_GRID);
        check.truncate(n);
        println!("Grid ({n}): {check}");
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

        let ll = grid_to_longlat(grid_in.as_str());
        assert!(ll.is_ok());

        // Make sure it's within the margin of error of the smallest field
        let (long, lat) = ll.unwrap();
        assert_delta!(long, TEST_LONG, LONG_MULT[n / 2 - 1]);
        assert_delta!(lat, TEST_LAT, LAT_MULT[n / 2 - 1]);

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
        assert!(ret.is_ok());
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
        println!("Distance: {dist} Bearing: {bear}");
        println!(
            "from: {:?} To: {:?}",
            grid_to_longlat("CM87um"),
            grid_to_longlat("KP04ow")
        );
        assert_delta!(dist, 8189.0, 1.0);
        assert_delta!(bear, 15.224, 0.001);
    }
}
