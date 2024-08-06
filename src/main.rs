use shapefile::{Point, PolygonRing, Shape};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let path = std::path::Path::new("data/polygon/polygon.shp");
  let mut reader = shapefile::Reader::from_path(path)?;

  for result in reader.iter_shapes_and_records() {
    let (shape, _record) = result?;

    let geojson = match shape {
      Shape::Polygon(polygon) => process_polygon(polygon)?,
      Shape::Polyline(polyline) => process_polyline(polyline)?,
      Shape::Point(point) => process_point(point)?,
      //Shape::MultiPoint(multipoint) => process_multipoint(multipoint)?,
      _ => {
        println!("Unsupported shape type");
        continue;
      }
    };

    println!("GeoJSON:\n{}", geojson);
  }

  Ok(())
}

fn process_polygon(polygon: shapefile::Polygon) -> Result<String, Box<dyn std::error::Error>> {
  let mut rings = Vec::new();
  for ring in polygon.rings() {
    let coordinates: Vec<Vec<f64>> = match ring {
      PolygonRing::Outer(points) | PolygonRing::Inner(points) => points
        .iter()
        .map(|point: &Point| vec![point.x, point.y])
        .collect(),
    };
    rings.push(coordinates);
  }

  let geometry = geojson::Geometry::new(geojson::Value::Polygon(rings));

  let feature = geojson::Feature {
    bbox: None,
    geometry: Some(geometry),
    id: None,
    properties: Some(serde_json::Map::new()),
    foreign_members: None,
  };

  let geojson_string = serde_json::to_string_pretty(&feature)?;
  Ok(geojson_string)
}

fn process_polyline(polyline: shapefile::Polyline) -> Result<String, Box<dyn std::error::Error>> {
  // Polyline の処理を実装
  Ok("Polyline processing not implemented".to_string())
}

fn process_point(point: shapefile::Point) -> Result<String, Box<dyn std::error::Error>> {
  // Point の処理を実装
  Ok("Point processing not implemented".to_string())
}

// fn process_multipoint(multipoint: shapefile::MultiPoint) -> Result<String, Box<dyn std::error::Error>> {
//     // MultiPoint の処理を実装
//     Ok("MultiPoint processing not implemented".to_string())
// }
