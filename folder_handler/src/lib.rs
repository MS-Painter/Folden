trait Handler{
    fn watch(); // Initalize handler to watch a folder
    fn generate_config(); // Generate configuration file with defaults to apply a new worker with it's own configuration
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
