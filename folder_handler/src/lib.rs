use std::{io, fs};
use std::path::Path;
use serde::Serialize;

trait Handler: Serialize {
    fn new() -> Self;
    fn from_bytes(bytes: Vec<u8>) -> Self;
    fn watch(&self); // Initialize handler to watch a folder
    fn generate_config<P: AsRef<Path>>(&self, path: P) -> io::Result<()>{ // Generate configuration file with defaults
        fs::write(path, toml::to_vec(&self).unwrap())
    }
}

pub mod common_handlers;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::any::{Any, TypeId};
    use crate::Handler;
    use crate::common_handlers::archive_join_handler::ArchiveJoinHandler;

    #[test]
    fn new_default_handler_works() {
        let file_path = "test_archive_join_handler_config.toml";
        let handler = ArchiveJoinHandler::new();
        handler.generate_config(file_path).unwrap();
    }

    #[test]
    fn handler_from_config_works() {
        let file_path = "test_archive_join_handler_config.toml";
        let file_bytes = &fs::read(file_path).unwrap();
        let handler = ArchiveJoinHandler::from_bytes(file_bytes.to_vec());
        assert_eq!(handler.type_id(), TypeId::of::<ArchiveJoinHandler>());
    }
}
