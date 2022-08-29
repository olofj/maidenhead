extern crate simple_error;
use self::simple_error::SimpleError;



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

static LONG_OFFSET: f32 = 180.0;
static LAT_OFFSET: f32 = 90.0;

static LONG_F: f32 = 20.0;
static LAT_F: f32 = 10.0;
static LONG_SQ: f32 = 2.0;
static LAT_SQ: f32 = 1.0;
static LONG_SSQ: f32 = 5.0 / 60.0;
static LAT_SSQ: f32 = 2.5 / 60.0;
static LONG_ESQ: f32 = 30.0 / 60.0 / 60.0;
static LAT_ESQ: f32 = 15.0 / 60.0 / 60.0;
static LONG_SESQ: f32 = 1.25 / 60.0 / 60.0;
static LAT_SESQ: f32 = 0.625 / 60.0 / 60.0;

static LONG_MULT: [f32; 5] = [LONG_F, LONG_SQ, LONG_SSQ, LONG_ESQ, LONG_SESQ];
static LAT_MULT: [f32; 5] = [LAT_F, LAT_SQ, LAT_SSQ, LAT_ESQ, LAT_SESQ];

pub fn grid_to_longlat(grid: &str) -> Result<(f32, f32), SimpleError>
{
    let vals: Vec<u32> = "AA00aa00AA".chars().zip(grid.chars()).map(
        | (t, c) | (c.to_ascii_uppercase() as u32) - (t.to_ascii_uppercase() as u32)).collect();
    let long: f32 = vals.iter().step_by(2).zip(LONG_MULT).map(| (&v, m) | v as f32 * m ).sum();
    let lat: f32 = vals.iter().skip(1).step_by(2).zip(LAT_MULT).map(| (&v, m) | v as f32 * m ).sum();
    Ok((long - LONG_OFFSET, lat - LAT_OFFSET))
}

pub fn longlat_to_grid(long: f32, lat: f32, precision: usize) -> Result<String, SimpleError>
{
    fn charoff(base: char, off: u32) -> Option<char> {
        std::char::from_u32(base as u32 + off)
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

    // It only makes sense to have 4+ even number of characters in a grid square
    match precision {
        0 | 4 | 6 | 8 | 10 => vals.truncate(precision),
        _ => return Err(SimpleError::new("Invalid grid length {precision}")),
    }

    let grid: Option<String> = "AA00aa00AA".chars().zip(vals).map(
        | (b, o) | charoff(b, o as u32) ).collect();
    match grid {
        Some(g) => Ok(g),
        None => Err(SimpleError::new("Failed to generate grid"))
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    // From https://stackoverflow.com/questions/30856285/assert-eq-with-floating-point-numbers-and-delta
    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !($x - $y < $d || $y - $x < $d) { panic!(); }
        };
    }

    static TEST_GRID: &str = &"FM18lv53SL";
    static TEST_LONG: f32 = -77.035278;
    static TEST_LAT: f32 = 38.889484;

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
    fn precision_4() {
        precision_n(4);
    }

    #[test]
    fn precision_inval() {
        let grid = longlat_to_grid(-77.035278, 38.889484, 5);
        assert!(grid.is_err());
    }

    fn longlat_n(n: usize) {
        let mut grid_in = String::from(TEST_GRID);
        grid_in.truncate(n);
        let ll = grid_to_longlat(&grid_in.as_str());
        assert!(!ll.is_err());
        let (long, lat) = ll.unwrap();
        println!("lat {} long {}", lat, long);
        // Make sure it's within the margin of error of the smallest field
        assert_delta!(long, -77.035278, LONG_MULT[4]);
        assert_delta!(lat, 38.889484, LAT_MULT[4]);
        let grid = longlat_to_grid(long, lat, n).unwrap();
        println!("grid_in {} grid {}", grid_in, grid);
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
}

