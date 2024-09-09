# Shapefile to GeoJSON Converter

This is a Rust library that provides a function to convert a shapefile (a common geospatial data format) to a GeoJSON file. GeoJSON is a lightweight, text-based format for representing geographic data, which is useful for a variety of mapping and data visualization applications.

## Example

This is a sample repository of how to use this library.

https://github.com/Tomoro0726/shapefile-to-geojson-ex

## Features

- Supports converting Polygon, Polyline, and Point geometries from Shapefile to GeoJSON
- Handles parsing of attribute data (fields) from the Shapefile's DBF file and includes them in the GeoJSON properties
- Uses a progress bar to provide feedback on the conversion process
- Supports asynchronous processing for improved performance

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

## Error Handling

The library uses a custom `CustomError` type to handle various errors that can occur during the conversion process, such as:

- `serde_json` errors when serializing the GeoJSON data
- `std::io` errors when reading or writing files
- Errors related to the Shapefile or DBF file formats

These errors are wrapped in a `Box<dyn std::error::Error + Send + Sync>` type, which allows the function to return a generic error type that can be handled by the caller.

## Multithreading and Asynchronous Processing

The library uses Tokio's asynchronous runtime and the `futures` crate to process the Shapefile data in parallel, leveraging multiple CPU cores for improved performance. The `process_shape_record` function is executed concurrently for each shape record in the Shapefile, with the results being accumulated in a `Vec<Feature>` that is then serialized to the GeoJSON output file.

## Progress Reporting

The library displays a progress bar using the `indicatif` crate, which provides a visual indication of the conversion progress. This can be helpful for larger Shapefiles, where the conversion process may take some time to complete.

## Dependencies

This library relies on the following Rust crates:

- `futures` for asynchronous processing
- `geojson` for GeoJSON data structures
- `indicatif` for progress bar visualization
- `regex` for parsing Shapefile attribute data
- `serde_json` for JSON serialization/deserialization
- `shapefile` for reading Shapefile data
- `tokio` for the async runtime
