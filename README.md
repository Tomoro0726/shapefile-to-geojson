# Shapefile to GeoJSON Converter

This is a Rust library that provides a function to convert a shapefile (a common geospatial data format) to a GeoJSON file. GeoJSON is a lightweight, text-based format for representing geographic data, which is useful for a variety of mapping and data visualization applications.

## Example

This is a sample repository of how to use this library.
https://github.com/Tomoro0726/shapefile-to-geojson-ex

## Usage

To use this library, you can call the `convert_shapefile_to_geojson` function, providing the input Shapefile path and the output GeoJSON file path:

```rust
use shapefile_to_geojson::convert_shapefile_to_geojson;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    convert_shapefile_to_geojson("input.shp", "output.geojson").await?;
    Ok(())
}
```

This function will read the Shapefile, process the geometric data, and write the resulting GeoJSON to the specified output file.

## Installation

To use this crate in your Rust project, add the following to your Cargo.toml file:

```toml
[dependencies]
shapefile-to-geojson = "*"
```

Then, import the crate in your Rust code:

```rust
use shapefile_to_geojson::convert_shapefile_to_geojson;
```

## Contributing

Contributions to this project are welcome! If you find any issues or have ideas for improvements, please feel free to open an issue or submit a pull request. We appreciate your help in making this crate better.
