use clap::App;

use folder_handler::handlers_json::HandlersJson;
use generated_types::inter_process_client::InterProcessClient;
use super::{generate_subcommand::GenerateSubCommand, register_subcommand::RegisterSubCommand, start_subcommand::StartSubCommand, status_subcommand::StatusSubCommand, stop_subcommand::StopSubCommand, subcommand::SubCommandUtil};

pub struct SubCommandCollection(Vec<Box<dyn SubCommandUtil>>);

impl SubCommandCollection {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, elem: Box<dyn SubCommandUtil>) {
        self.0.push(elem);
    }

    pub fn collect_as_apps(&self) -> Vec<App> {
        self.0.as_slice().into_iter()
        .map(|item| item.construct_subcommand())
        .collect()
    }
}

impl IntoIterator for SubCommandCollection {
    type Item = Box<dyn SubCommandUtil>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct Cli<'a, 'b> {
    pub base_app: App<'a, 'b>,
    pub subcommands: SubCommandCollection 
}

impl<'a, 'b> Cli<'a, 'b> {
    pub fn new(base_app: App<'a,'b>) -> Self {
        let mut subcommands = SubCommandCollection::new();
        let handlers_json = HandlersJson::new();
        subcommands.add(Box::new(RegisterSubCommand {
            handlers_json: handlers_json.clone(),        
        }));
        subcommands.add(Box::new(StatusSubCommand {}));
        subcommands.add(Box::new(StartSubCommand {}));
        subcommands.add(Box::new(StopSubCommand {}));
        subcommands.add(Box::new(GenerateSubCommand {
            handlers_json,        
        }));
        Self { base_app, subcommands}
    }

    pub fn construct_app(&mut self) -> App {
        self.base_app.clone().subcommands(self.subcommands.collect_as_apps())
    }

    pub fn execute(&mut self) {
        
        for subcommand in self.subcommands.into_iter() {           
        }
    }
}

pub fn excute(mut cli: Cli, url: &str)
 {
    let client_connect_future = InterProcessClient::connect(url);
    let matches = cli.construct_app().get_matches();
    for subcommand in cli.subcommands {
        if let Some(sub_matches) = subcommand.subcommand_matches(&matches) {
            subcommand.subcommand_runtime(sub_matches, client_connect_future);
            // Ok(())
        }
    }
    // Err("No subcommand matched")
}