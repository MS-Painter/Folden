trait Handler{
    fn watch(); // Initialize handler to watch a folder
    fn generate_config(); // Generate configuration file with defaults
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
