use std::{fs, ops::Deref, sync::Arc};

use tokio::sync::RwLock;

use crate::config::Config;
use crate::mapping::Mapping;
use folder_handler::handlers_json::HandlersJson;

pub mod inter_process;

pub struct Server {
    pub config: Arc<Config>,
    pub mapping: Arc<RwLock<Mapping>>,
    pub handlers_json: Arc<HandlersJson>,
}

impl Server {
    pub async fn save_mapping(&self) -> Result<(), std::io::Error> {
        let mapping = self.mapping.read().await;
        let mapping = mapping.deref();
        let mapping_data: Vec<u8> = mapping.into();
        fs::write(&self.config.mapping_state_path, mapping_data)
    }
}
