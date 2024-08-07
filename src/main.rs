use futures::stream::{self, StreamExt};
use geojson::{Feature, FeatureCollection};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde_json::json;
use shapefile::{PolygonRing, Reader, Shape};
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
struct CustomError(String);

impl fmt::Display for CustomError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::error::Error for CustomError {}

impl From<serde_json::Error> for CustomError {
  fn from(err: serde_json::Error) -> Self {
    CustomError(err.to_string())
  }
}

impl From<std::io::Error> for CustomError {
  fn from(err: std::io::Error) -> Self {
    CustomError(err.to_string())
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let base_path = Path::new("data/polygon/polygon");
  let shp_path = base_path.with_extension("shp");
  let dbf_path = base_path.with_extension("dbf");
  let mut shp_reader = Reader::from_path(shp_path.clone())?;
  let mut all_count: u64 = 0;
  {
    let mut shp_reader = Reader::from_path(&shp_path)?;
    let mut dbf_reader = dbase::Reader::from_path(&dbf_path)?;
    let shp_count = &shp_reader.iter_shapes_and_records().count();
    let dbf_count = &dbf_reader.iter_records().count();

    all_count = *shp_count as u64;

    if shp_count != dbf_count {
      println!("Warning: SHP data ({} records) and DBF data ({} records) have different numbers of elements.", shp_count, dbf_count);
    } else {
      println!(
        "SHP and DBF data have the same number of elements: {} records",
        shp_count
      );
    }
  }

  let features = Arc::new(Mutex::new(Vec::new()));

  let pb = ProgressBar::new(all_count);
  pb.set_style(
    ProgressStyle::default_bar()
      .template("{spinner:.green} [{bar:40.cyan/blue}] {msg}")?
      .progress_chars("█▓▒░"),
  );
  pb.set_message("進行中...");

  let shape_records: Vec<_> = shp_reader.iter_shapes_and_records().collect();

  let tasks = stream::iter(shape_records)
    .map(|shape_record| {
      let features = Arc::clone(&features);
      let pb = pb.clone();

      tokio::spawn(async move {
        let (shape, record) = shape_record?;
        let geojson_string = match shape {
          Shape::Polygon(_) => process_polygon(&shape)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?,
          Shape::Polyline(_) => process_polyline(&shape)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?,
          Shape::Point(_) => process_point(&shape)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?,
          _ => return Ok(()), // Skip unsupported shapes
        };

        let mut feature: Feature = serde_json::from_str(&geojson_string)?;

        if let Some(props) = &mut feature.properties {
          let re_numeric = Regex::new(r"Numeric\(Some\(([0-9]+(\.[0-9]+)?)\)\)").unwrap();
          let re_character = Regex::new(r#"^Character\(Some\("(.+)"\)\)"#).unwrap();

          for (field, value) in record.into_iter() {
            let value_string = value.to_string();
            props.insert(field.to_string(), json!(value_string));
          }
        }

        features.lock().await.push(feature);
        pb.inc(1);
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
      })
    })
    .buffer_unordered(num_cpus::get());

  tasks.for_each(|_| async {}).await;

  println!("完了");
  pb.finish_with_message("完了");

  let feature_collection = FeatureCollection {
    bbox: None,
    features: features.lock().await.clone(),
    foreign_members: None,
  };

  let geojson_output = serde_json::to_string_pretty(&feature_collection)?;

  let mut file = File::create("output.geojson")?;
  file.write_all(geojson_output.as_bytes())?;

  println!("GeoJSON file has been created: output.geojson");

  Ok(())
}

fn process_polygon(shape: &Shape) -> Result<String, CustomError> {
  let polygon = match shape {
    Shape::Polygon(p) => p,
    _ => return Err(CustomError("Expected Polygon shape".to_string())),
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

  serde_json::to_string(&feature).map_err(CustomError::from)
}

fn process_polyline(shape: &Shape) -> Result<String, CustomError> {
  let polyline = match shape {
    Shape::Polyline(p) => p,
    _ => return Err(CustomError("Expected Polyline shape".to_string())),
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

  serde_json::to_string(&feature).map_err(CustomError::from)
}

fn process_point(shape: &Shape) -> Result<String, CustomError> {
  let point = match shape {
    Shape::Point(p) => p,
    _ => return Err(CustomError("Expected Point shape".to_string())),
  };

  let feature = json!({
      "type": "Feature",
      "geometry": {
          "type": "Point",
          "coordinates": [point.x, point.y]
      },
      "properties": null
  });

  serde_json::to_string(&feature).map_err(CustomError::from)
}
