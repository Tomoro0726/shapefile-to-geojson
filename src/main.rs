use dbase;
use shapefile::Reader;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let shp_path = "data/line/line.shp";
  let dbf_path = "data/line/line.dbf";

  // ファイルの存在確認
  if !Path::new(shp_path).exists() {
    return Err(format!("SHP file does not exist: {}", shp_path).into());
  }
  if !Path::new(dbf_path).exists() {
    return Err(format!("DBF file does not exist: {}", dbf_path).into());
  }

  // SHPリーダーの作成
  let mut shp_reader = Reader::from_path(shp_path)
    .map_err(|e| format!("Failed to create SHP reader for '{}': {}", shp_path, e))?;

  // DBFリーダーの作成
  let mut dbf_reader = dbase::Reader::from_path(dbf_path)
    .map_err(|e| format!("Failed to create DBF reader for '{}': {}", dbf_path, e))?;

  // レコード数の比較
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
  let shape_type = shp_reader.header().shape_type;
  println!("Shape type: {:?}", shape_type);

  Ok(())
}
