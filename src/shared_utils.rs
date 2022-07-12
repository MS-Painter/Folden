use clap::Arg;

use crate::shared_config::DEFAULT_PORT;

pub fn construct_port_arg<'a>() -> Arg<'a> {
    Arg::with_name("port")
        .short('p')
        .long("port")
        .default_value(DEFAULT_PORT)
        .empty_values(false)
        .takes_value(true)
}
