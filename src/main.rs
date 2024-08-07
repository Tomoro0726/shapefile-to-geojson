use geojson::{Feature, FeatureCollection, Geometry, Value as GeoJsonValue};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde_json::{json, Map};
use shapefile::{Point, PolygonRing, Reader, Shape};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let base_path = Path::new("data/polygon/polygon");
  let shp_path = base_path.with_extension("shp");
  let dbf_path = base_path.with_extension("dbf");
  let mut shp_reader = Reader::from_path(shp_path.clone())?;
  let mut all_count: usize = 0;
  {
    let mut shp_reader = Reader::from_path(shp_path)?;
    let mut dbf_reader = dbase::Reader::from_path(dbf_path)?;
    let shp_count = &shp_reader.iter_shapes_and_records().count();
    let dbf_count = &dbf_reader.iter_records().count();

    all_count = shp_count.clone();

    if shp_count != dbf_count {
      println!("Warning: SHP data ({} records) and DBF data ({} records) have different numbers of elements.", shp_count, dbf_count);
    } else {
      println!(
        "SHP and DBF data have the same number of elements: {} records",
        shp_count
      );
    }
  }
  let mut features = Vec::new();

  //ステータスバー
  let pb = ProgressBar::new(all_count as u64);
  pb.set_style(
    ProgressStyle::default_bar()
      .template("{spinner:.green} [{bar:40.cyan/blue}] {msg}")? //記号や文字の色を変えるよ！
      .progress_chars("█▓▒░"),
  ); //好みに変更してね！2つ以上の文字を入れてね
  pb.set_message("進行中...");
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
        let re_1 = Regex::new(&format!("^{}", "Numeric")).unwrap();
        let re_2 = Regex::new(&format!("^{}", "Numeric")).unwrap();
        if re_1.is_match(&value.to_string()) {
          props.insert(
            field.to_string(),
            json!(remove_non_numeric(&value.to_string())),
          );
        } else if re_2.is_match(&value.to_string()) {
          props.insert(
            field.to_string(),
            json!(remove_non_numeric(&value.to_string())),
          );
        } else {
          props.insert(field.to_string(), json!(value.to_string()));
        }
      }
    }

    features.push(feature);

    pb.inc(1);
  }
  println!("完了");
  pb.finish_with_message("完了");

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
  //println!("process_polygonが実行されました。");
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

  let properties: Map<String, serde_json::Value> = Map::new();

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
  //println!("process_polylineが実行されました。");
  let polyline = match shape {
    Shape::Polyline(p) => p,
    _ => return Err("Expected Polyline shape".into()),
  };

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
  //println!("process_pointが実行されました。");
  let point = match shape {
    Shape::Point(p) => p,
    _ => return Err("Expected Point shape".into()),
  };

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

fn remove_non_numeric(text: &String) -> String {
  let re = Regex::new(r"\D").unwrap();
  let result = re.replace_all(text, String::new());
  result.into_owned() // Convert Cow<'_, str> to String
}
