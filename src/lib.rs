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
pub struct CustomError(String);

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

pub async fn convert_shapefile_to_geojson(
  input_path: &str,
  output_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let base_path = Path::new(input_path);
  let shp_path = base_path.with_extension("shp");
  let dbf_path = base_path.with_extension("dbf");
  let mut shp_reader = Reader::from_path(shp_path.clone())?;
  let all_count = count_records(&shp_path, &dbf_path)?;

  let features = Arc::new(Mutex::new(Vec::new()));

  let pb = create_progress_bar(all_count);

  let shape_records: Vec<_> = shp_reader.iter_shapes_and_records().collect();

  let tasks = stream::iter(shape_records)
    .map(|shape_record| {
      let features = Arc::clone(&features);
      let pb = pb.clone();

      tokio::spawn(async move { process_shape_record(shape_record, features, pb).await })
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

  let mut file = File::create(output_path)?;
  file.write_all(geojson_output.as_bytes())?;

  println!("GeoJSON file has been created: {}", output_path);

  Ok(())
}

fn count_records(
  shp_path: &Path,
  dbf_path: &Path,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
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

  Ok(shp_count as u64)
}

fn create_progress_bar(total: u64) -> ProgressBar {
  let pb = ProgressBar::new(total);
  pb.set_style(
    ProgressStyle::default_bar()
      .template("{spinner:.green} [{bar:40.cyan/blue}] {msg}")
      .unwrap()
      .progress_chars("█▓▒░"),
  );
  pb.set_message("進行中...");
  pb
}

async fn process_shape_record(
  shape_record: Result<(Shape, shapefile::dbase::Record), shapefile::Error>,
  features: Arc<Mutex<Vec<Feature>>>,
  pb: ProgressBar,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let (shape, record) = shape_record?;
  let geojson_string = match shape {
    Shape::Polygon(_) => process_polygon(&shape)?,
    Shape::Polyline(_) => process_polyline(&shape)?,
    Shape::Point(_) => process_point(&shape)?,
    _ => return Ok(()), // Skip unsupported shapes
  };

  let mut feature: Feature = serde_json::from_str(&geojson_string)?;

  if let Some(props) = &mut feature.properties {
    let re_numeric = Regex::new(r"Numeric\(Some\(([0-9]+(\.[0-9]+)?)\)\)").unwrap();
    let re_character = Regex::new(r#"^Character\(Some\("(.+)"\)\)"#).unwrap();

    for (field, value) in record.into_iter() {
      let value_string = value.to_string();
      let value_json = match value_string.as_str() {
        "Numeric(None)" => json!(""),
        "Character(None)" => json!(null),
        _ => {
          if let Some(caps) = re_numeric.captures(&value_string) {
            let number_str = caps.get(1).map_or("", |m| m.as_str());
            if let Ok(number) = number_str.parse::<f64>() {
              json!(number)
            } else {
              eprintln!("Failed to parse numeric value: {}", value_string);
              json!(value_string)
            }
          } else if let Some(caps) = re_character.captures(&value_string) {
            let character_str = caps.get(1).map_or("", |m| m.as_str());
            json!(character_str)
          } else {
            json!(value_string)
          }
        }
      };
      props.insert(field.to_string(), value_json);
    }
  }

  features.lock().await.push(feature);
  pb.inc(1);
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
      "properties": {}
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
      "properties": {}
  });

  serde_json::to_string(&feature).map_err(CustomError::from)
}
