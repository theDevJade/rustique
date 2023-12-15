use std::{env, path::PathBuf};

use cliclack::{confirm, intro, outro};
use config::Config;
use miette::Result;
use server_gen::api_util::ServerManager;

pub mod server_gen;
pub mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let action: &str = args[1].as_str();

    match action {
        "help" => help().await,
        "generate" => {
            let _ = server_gen::generate().await;
        },
        "list" => {
            let _ = Config::display_servers();
        },
        "run" => {
            let name = args.get(2).unwrap();
            let config = Config::read_from_file(Config::default_config_path()).expect("Invalid config");
            let server = Config::get_server(&config, name).unwrap();
            let _ = ServerManager::run_server(&ServerManager::new("paper", "paper").await.unwrap(), &PathBuf::from(server.clone().path)).await;
        },
        "delete" => {
            let _ = intro("Server Deletion");
            let confirm = confirm("Are you sure you want to delete this?").interact().unwrap();
            if (confirm) {
                let name = args.get(2).unwrap();
                let mut config = Config::read_from_file(Config::default_config_path()).expect("Invalid config");
                let server = Config::get_server(&mut config, name).unwrap();
                let mut config2 = Config::read_from_file(Config::default_config_path()).expect("Invalid config");
                config2.remove_server(server);
                let _ = config2.save_to_file(Config::default_config_path());
               let _ = ServerManager::delete_server(&PathBuf::from(server.clone().path)).await;
               let _ = outro("Server Deleted");
            } else {
                let _ = outro("Cancelled Deletion");
            }

            
            
        },
        _ => {
            help().await
        }
    }
    Ok(())
}

async fn help() {
    println!("
    📦Rustique Help📦
    ⎯⎯⎯⎯⎯⎯⎯⎯

    help - Display this help message
    generate - Generate a new server with prompts
    list - List all servers added to Rustique
    run <server> - Run the specified server
    delete <server> - Delete the specified server

    ⎯⎯⎯⎯⎯⎯⎯⎯
    Made by Jade with ❤️
    ");
}
