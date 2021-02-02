use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub db_path: String, // Local db file path
    pub continue_on_startup: bool, // On server startup, should continue working on previous handled directories? Saves this data to mapping file 
}

impl From<Vec<u8>> for Config {
    fn from(bytes: Vec<u8>) -> Self {
        toml::from_slice(&bytes).unwrap()
    }
}