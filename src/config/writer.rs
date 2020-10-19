use std::io::Write;

use yaml_rust::YamlEmitter;
use yaml_rust::Yaml;

use super::super::errors;


/// Write a Yaml object to a file

pub fn write_yaml<P>(doc: &Yaml, path: P) -> Result<(), errors::GardenError>
where
    P: std::convert::AsRef<std::path::Path> + std::fmt::Debug,
{
    // Emit the YAML configuration into a string
    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&doc).ok(); // dump the YAML object to a String
    }
    out_str += "\n";

    let mut file = std::fs::File::create(&path).map_err(|io_err| {
        errors::GardenError::CreateConfigurationError {
            path: path.as_ref().into(),
            err: io_err,
        }
    })?;

    file.write_all(&out_str.into_bytes()).map_err(|_| {
        errors::GardenError::WriteConfigurationError { path: path.as_ref().into() }
    })?;

    file.sync_all().map_err(|sync_err| {
        errors::GardenError::SyncConfigurationError {
            path: path.as_ref().into(),
            err: sync_err,
        }
    })
}
