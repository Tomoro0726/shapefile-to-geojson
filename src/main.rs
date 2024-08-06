use dbase;
use shapefile::Reader;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let shp_path = "C:/Users/tomor/Downloads/500m_mesh_suikei_2018_shape_13/500m_mesh_2018_13.shp";
  let dbf_path = "C:/Users/tomor/Downloads/500m_mesh_suikei_2018_shape_13/500m_mesh_2018_13.dbf";

  // ファイルの存在確認と読み取り可能性のチェック
  check_file(shp_path)?;
  check_file(dbf_path)?;

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

  // シェイプタイプの表示
  let shape_type = shp_reader.header().shape_type.to_string();
  println!("Shape type: {}", shape_type);

  // シェイプタイプごとの変換処理
  if shape_type == "Polygon" {
    for dbf_record in dbf_reader.iter_records() {
      match dbf_record {
        Ok(record) => println!("{:?}", record),
        Err(e) => println!("Error reading DBF record: {:?}", e),
      }
    }

    for shp_record in shp_reader.iter_shapes_and_records() {
      match shp_record {
        Ok((shape, record)) => println!("Geometry: {}, Properties {:?}", shape, record),
        Err(e) => println!("Error reading SHP record: {:?}", e),
      }
    }
  } else {
    println!("Type:{} is not supported", shape_type);
  }

  Ok(())
}

fn check_file(path: &str) -> io::Result<()> {
  let mut file = File::open(path)?;
  let mut buffer = [0; 1024];

  // ファイルの先頭を読み取る
  file.read(&mut buffer)?;

  // ファイルの終わりまでシーク
  file.seek(SeekFrom::End(0))?;

  Ok(())
}
