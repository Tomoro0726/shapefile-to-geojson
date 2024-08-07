use geojson::{Feature, FeatureCollection, Geometry, Value as GeoJsonValue};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde_json::{json, Map};
use shapefile::{Point, PolygonRing, Reader, Shape};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let base_path = Path::new("data/line/line");
  let shp_path = base_path.with_extension("shp");
  let dbf_path = base_path.with_extension("dbf");
  let mut shp_reader = Reader::from_path(shp_path.clone())?;
  let mut all_count: u64 = 0;
  {
    let mut shp_reader = Reader::from_path(shp_path)?;
    let mut dbf_reader = dbase::Reader::from_path(dbf_path)?;
    let shp_count = &shp_reader.iter_shapes_and_records().count();
    let dbf_count = &dbf_reader.iter_records().count();

    all_count = shp_count.clone() as u64;

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
  let pb = ProgressBar::new(all_count);
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
        let re_numeric = Regex::new(r"^Numeric").unwrap();
        let re_character = Regex::new(r#"^Character\(Some\("(.+)"\)\)"#).unwrap();

        if re_numeric.is_match(&value.to_string()) {
          let numeric_string = remove_non_numeric(&value.to_string());
          if let Ok(number) = numeric_string.parse::<f64>() {
            props.insert(field.to_string(), json!(number));
          } else {
            props.insert(field.to_string(), json!(numeric_string));
          }
        } else if let Some(captures) = re_character.captures(&value.to_string()) {
          if let Some(inner_value) = captures.get(1) {
            props.insert(field.to_string(), json!(inner_value.as_str()));
          } else {
            props.insert(field.to_string(), json!(value.to_string()));
          }
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
  let polygon = match shape {
    Shape::Polygon(p) => p,
    _ => return Err("Expected Polygon shape".into()),
  };

  let rings: Vec<Vec<Vec<f64>>> = polygon
    .rings()
    .iter()
    .map(|ring| match ring {
      PolygonRing::Outer(points) | PolygonRing::Inner(points) => {
        points.iter().map(|point| vec![point.x, point.y]).collect()
      }
    })
    .collect();

  let feature = json!({
      "type": "Feature",
      "geometry": {
          "type": "Polygon",
          "coordinates": rings
      },
      "properties": {}
  });

  serde_json::to_string(&feature).map_err(Into::into)
}
fn process_polyline(shape: &Shape) -> Result<String, Box<dyn std::error::Error>> {
  let polyline = match shape {
    Shape::Polyline(p) => p,
    _ => return Err("Expected Polyline shape".into()),
  };

  let parts: Vec<Vec<Vec<f64>>> = polyline
    .parts()
    .iter()
    .map(|part| part.iter().map(|point| vec![point.x, point.y]).collect())
    .collect();

  let feature = json!({
      "type": "Feature",
      "geometry": {
          "type": "MultiLineString",
          "coordinates": parts
      },
      "properties": null
  });

  serde_json::to_string(&feature).map_err(Into::into)
}

fn process_point(shape: &Shape) -> Result<String, Box<dyn std::error::Error>> {
  let point = match shape {
    Shape::Point(p) => p,
    _ => return Err("Expected Point shape".into()),
  };

  let feature = json!({
      "type": "Feature",
      "geometry": {
          "type": "Point",
          "coordinates": [point.x, point.y]
      },
      "properties": null
  });

  serde_json::to_string(&feature).map_err(Into::into)
}

fn remove_non_numeric(text: &String) -> String {
  let re = Regex::new(r"\D").unwrap();
  let result = re.replace_all(text, String::new());
  result.into_owned() // Convert Cow<'_, str> to String
}
