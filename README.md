# Maidenhead

A simple Rust library for converting between Maidenhead grid squares
and latitude/longitude coordinates.

## Overview

Maidenhead grid squares are a coordinate system used by amateur radio
operators to specify locations on Earth. This library provides functions
to convert between grid squares and decimal degree coordinates, as well
as calculate distances and bearings between grid squares.

## Features

- Convert grid squares to latitude/longitude coordinates
- Convert latitude/longitude coordinates to grid squares (4, 6, 8, or 10 character precision)
- Calculate distance between two grid squares using the Haversine formula
- Calculate bearing between two grid squares
- Comprehensive error handling with descriptive error messages

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
maidenhead = "0.1.1"
```

### Examples

```rust
use maidenhead::*;

// Convert grid square to coordinates
let (longitude, latitude) = grid_to_longlat("FM18lv")?;
println!("Coordinates: {}, {}", longitude, latitude);

// Convert coordinates to grid square
let grid = longlat_to_grid(-77.035278, 38.889484, 6)?;
println!("Grid square: {}", grid);

// Calculate distance between two grid squares
let distance = grid_distance("FM18lv", "EN91")?;
println!("Distance: {:.2} km", distance);

// Calculate bearing between two grid squares
let bearing = grid_bearing("FM18lv", "EN91")?;
println!("Bearing: {:.1}°", bearing);

// Calculate both distance and bearing
let (distance, bearing) = grid_dist_bearing("FM18lv", "EN91")?;
println!("Distance: {:.2} km, Bearing: {:.1}°", distance, bearing);
```

## Grid Square Format

Grid squares follow the format: `FFSSssEEee` where:
- **FF**: Field (2 letters, A-R) - covers 20°/10° longitude/latitude
- **SS**: Square (2 digits, 0-9) - covers 2°/1° longitude/latitude  
- **ss**: Subsquare (2 letters, a-x) - covers 5'/2.5' longitude/latitude
- **EE**: Extended square (2 digits, 0-9) - covers 30"/15" longitude/latitude
- **ee**: Superextended square (2 letters, A-X) - covers 1.25"/0.625" longitude/latitude

Supported precisions: 4, 6, 8, or 10 characters.

## Error Handling

The library uses the `thiserror` crate for comprehensive error handling:

- `InvalidGrid`: Invalid grid square format
- `InvalidGridLength`: Unsupported grid square length
- `InvalidLongLat`: Coordinates out of valid range
- `Unknown`: Internal error during grid generation

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
