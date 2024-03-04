use std::fs::Metadata;
use std::path::Path;
use std::io::ErrorKind;

pub fn get_metadata(path: &Path) -> Result<Option<Metadata>, std::io::Error> {
    match path.metadata() {
        Ok(m) => Ok(Some(m)),
        Err(err) => match err.kind() {
            ErrorKind::NotFound => Ok(None),
            _ => Err(err),
        }
    }
}
