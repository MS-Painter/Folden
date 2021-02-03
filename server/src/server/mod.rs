use std::sync::Arc;

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
