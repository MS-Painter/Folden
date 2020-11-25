#[typetag::serde(tag = "type")]
trait Handler {
    fn watch(&self); // Initialize handler to watch a folder
}

pub mod handlers_json {
    use serde_json::Value;
    use crate::common_handlers::archive_join_handler::ArchiveJoinHandler;
    use crate::Handler;

    pub struct HandlersJson{
        pub(crate) handlers: Vec<Value>
    }

    impl HandlersJson {
        pub fn new() -> Self {
            let archive_join_handler = ArchiveJoinHandler::default();
            let dyn_archive_join_handler = &archive_join_handler as &dyn Handler;
            let dyn_handlers = vec![dyn_archive_join_handler];
            Self{
                handlers: dyn_handlers.iter().map(|handler| {
                    let json = serde_json::to_string(handler).unwrap();
                    serde_json::from_str(&*json).unwrap()
                }).collect()
            }
        }

        pub fn get_handler_types(&self) -> Vec<&str> {
            self.handlers.iter().map(|handler| handler["type"].as_str().unwrap()).collect()
        }
    }
}


pub mod common_handlers;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::any::{TypeId, Any};
    use crate::common_handlers::archive_join_handler::ArchiveJoinHandler;
    use crate::handlers_json::HandlersJson;

    #[test]
    fn typetagging_works() {
        let handlers_json = HandlersJson::new();
        println!("PageLoad json len: {}", handlers_json.get_handler_types().len());
        println!("PageLoad json: {}", handlers_json.handlers[0]);
        println!("PageLoad json: {}", handlers_json.handlers[0]["type"]);
    }

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
