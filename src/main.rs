use geojson::{Feature, FeatureCollection, Geometry, Value as GeoJsonValue};
use serde_json::{json, Map};
use shapefile::{Point, PolygonRing, Reader, Shape};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let base_path = Path::new("data/polygon/polygon");
  let shp_path = base_path.with_extension("shp");
  let dbf_path = base_path.with_extension("dbf");
  let mut shp_reader = Reader::from_path(shp_path)?;
  let mut dbf_reader = dbase::Reader::from_path(dbf_path)?;

  let shp_count = shp_reader.iter_shapes_and_records().count();
  let dbf_count = dbf_reader.iter_records().count();

  if shp_count != dbf_count {
    println!("Warning: SHP data ({} records) and DBF data ({} records) have different numbers of elements.", shp_count, dbf_count);
  } else {
    println!(
      "SHP and DBF data have the same number of elements: {} records",
      shp_count
    );
  }

  let mut features = Vec::new();

  for shape_record in shp_reader.iter_shapes_and_records() {
    let (shape, record) = shape_record?;
    let geojson_string = match shape {
      Shape::Polygon(_) => process_polygon(&shape)?,
      Shape::Polyline(_) => process_polyline(&shape)?,
      Shape::Point(_) => process_point(&shape)?,
      _ => continue, // Skip unsupported shapes
    };

    let mut feature: Feature = serde_json::from_str(&geojson_string)?;

    // Add properties from the DBF record
    if let Some(props) = &mut feature.properties {
      for (field, value) in record.into_iter() {
        props.insert(field.to_string(), json!(value.to_string()));
      }
    }

    features.push(feature);
  }

  let feature_collection = FeatureCollection {
    bbox: None,
    features,
    foreign_members: None,
  };

  let geojson_output = serde_json::to_string_pretty(&feature_collection)?;

  // Write the GeoJSON to a file
  let mut file = File::create("output.geojson")?;
  file.write_all(geojson_output.as_bytes())?;

  println!("GeoJSON file has been created: output.geojson");

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
