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
            Self::InvalidGridLength(len) => {
                write!(f, "Invalid grid length {len}, only 4/6/8/10 supported")
            }
            Self::InvalidLongLat(long, lat) => {
                write!(f, "Invalid Longitude/Latitude: `{long}`/`{lat}`")
            }
            Self::Unknown => write!(f, "unknown error when generating grid string"),
        }
    }
}

impl Error for MHError {}
