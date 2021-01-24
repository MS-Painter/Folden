use std::sync::Arc;

use rocksdb::DB;

use crate::config::Config;

pub mod inter_process;

pub struct Server {
    pub config: Arc<Config>,
    pub db: Arc<DB>
}