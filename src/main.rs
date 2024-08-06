use geojson::{Feature, Geometry, Value as GeoJsonValue};
use serde_json::{json, Map};
use shapefile::{Point, PolygonRing, Reader, Shape};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let path = std::path::Path::new("data/polygon/polygon.shp");
  let mut reader = Reader::from_path(path)?;

  for shape_record in reader.iter_shapes_and_records() {
    let (shape, _) = shape_record?;

    let geojson = match shape {
      Shape::Polygon(_) => process_polygon(&shape)?,
      Shape::Polyline(_) => process_polyline(&shape)?,
      Shape::Point(_) => process_point(&shape)?,
      _ => {
        println!("Unsupported shape type");
        continue;
      }
    };

    println!("GeoJSON:\n{}", geojson);
  }

  Ok(())
}

fn process_polygon(shape: &Shape) -> Result<String, Box<dyn std::error::Error>> {
  let polygon = match shape {
    Shape::Polygon(p) => p,
    _ => return Err("Expected Polygon shape".into()),
  };
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

  let mut properties: Map<String, serde_json::Value> = Map::new();
  properties.insert("name".to_string(), json!("My Feature"));
  properties.insert("population".to_string(), json!(12345));

  let geometry = Geometry::new(GeoJsonValue::Polygon(rings));
  let feature = Feature {
    bbox: None,
    geometry: Some(geometry),
    properties: Some(properties),
    id: None,
    foreign_members: None,
  };

  let geojson_string = serde_json::to_string_pretty(&feature)?;

  Ok(geojson_string)
}

fn process_polyline(shape: &Shape) -> Result<String, Box<dyn std::error::Error>> {
  let polyline = match shape {
    Shape::Polyline(p) => p,
    _ => return Err("Expected Polyline shape".into()),
  };

  println!("Processing Polyline:");
  println!("Number of parts: {}", polyline.parts().len());
  println!(
    "Total number of points: {}",
    polyline
      .parts()
      .iter()
      .map(|part| part.len())
      .sum::<usize>()
  );

  let mut parts = Vec::new();
  for part in polyline.parts() {
    let coordinates: Vec<Vec<f64>> = part
      .iter()
      .map(|point: &Point| vec![point.x, point.y])
      .collect();
    parts.push(coordinates);
  }

  let geometry = Geometry::new(GeoJsonValue::MultiLineString(parts));

  let feature = Feature {
    bbox: None,
    geometry: Some(geometry),
    id: None,
    properties: None,
    foreign_members: None,
  };

  let geojson_string = serde_json::to_string_pretty(&feature)?;

  Ok(geojson_string)
}

fn process_point(shape: &Shape) -> Result<String, Box<dyn std::error::Error>> {
  let point = match shape {
    Shape::Point(p) => p,
    _ => return Err("Expected Point shape".into()),
  };

  println!("Processing Point:");
  println!("Coordinates: ({}, {})", point.x, point.y);

  let geometry = Geometry::new(GeoJsonValue::Point(vec![point.x, point.y]));

  let feature = Feature {
    bbox: None,
    geometry: Some(geometry),
    properties: None,
    id: None,
    foreign_members: None,
  };

  let geojson_string = serde_json::to_string_pretty(&feature)?;

  Ok(geojson_string)
}
