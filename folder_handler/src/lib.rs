#[typetag::serde(tag = "type")]
trait Handler {
    fn watch(&self); // Initialize handler to watch a folder
}

pub mod common_handlers;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::any::{TypeId, Any};
    use crate::common_handlers::archive_join_handler::ArchiveJoinHandler;

    #[test]
    fn new_default_handler_works() {
        let file_path = "test_archive_join_handler_config.toml";
        let handler = ArchiveJoinHandler::default();
        handler.generate_config(file_path.as_ref()).unwrap();
    }

    #[test]
    fn handler_from_config_works() {
        let file_path = "test_archive_join_handler_config.toml";
        let file_bytes = &fs::read(file_path).unwrap();
        let handler = ArchiveJoinHandler::from(file_bytes.to_vec());
        assert_eq!(handler.type_id(), TypeId::of::<ArchiveJoinHandler>());
    }
}
