use clap::{App, AppSettings, Arg};

const DEFAULT_CONFIG_PATH: &str = "default.conf";

fn main() {
    let app = App::new("Folden Server")
        .version("0.1")
        .about("Folden background manager")
        .setting(AppSettings::ColorAuto)
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true));
    let matches = app.get_matches();
    let config = matches.value_of("config").unwrap_or(DEFAULT_CONFIG_PATH);
    println!("Value for config: {}", config);
}
