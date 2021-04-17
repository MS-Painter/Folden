use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::Config;
use crate::mapping::Mapping;

pub mod inter_process;

pub struct Server {
    pub config: Arc<Config>,
    pub mapping: Arc<RwLock<Mapping>>,
}