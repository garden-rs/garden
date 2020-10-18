use std::io::Write;

//TODO
//#[macro_use]
//use anyhow::Context;
use yaml_rust::YamlEmitter;
use yaml_rust::Yaml;

use super::super::errors;


/// Write a Yaml object to a file

pub fn write_yaml<P>(doc: &Yaml, path: P) -> Result<(), errors::GardenError>
where P: std::convert::AsRef<std::path::Path> + std::fmt::Debug {
    // Emit the YAML configuration into a string
    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&doc).ok(); // dump the YAML object to a String
    }
    out_str += "\n";

    let file_result = std::fs::File::create(&path);
    if file_result.is_err() {
        return Err(
            errors::GardenError::CreateConfigurationError {
                path: path.as_ref().into(),
                err: file_result.err().unwrap(),
            }.into()
        );
    }

    let mut file = file_result.unwrap();
    let write_result = file.write_all(&out_str.into_bytes());
    if write_result.is_err() {
        return Err(
            errors::GardenError::WriteConfigurationError {
                path: path.as_ref().into(),
            }.into()
        );
    }

    if let Err(err) = file.sync_all() {
        return Err(
            errors::GardenError::SyncConfigurationError {
                path: path.as_ref().into(),
                err: err,
            }.into()
        );
    }

    Ok(())
}
