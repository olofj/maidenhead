use maidenhead::*;

fn main() -> Result<(), MHError>{
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

    // With feature `structs`
    let grid1 = GridSquare::new("jo30uj")?;
    println!("Grid square: {}", grid1);
    let coord1: Coordinate = grid1.into();
    println!("As coordinate: {}", coord1);
    let grid2 = GridSquare::new("JO30ui")?;
    println!("{}", coord1 - grid2.into());
    
    Ok(())
}