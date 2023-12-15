use std::path::PathBuf;

use cliclack::*;
use colored::Colorize;

use self::api_util::ServerManager;
use crate::config::{Config, Server};
pub mod api_util;


pub async fn generate() -> Result<(), Box<dyn std::error::Error>> {
    intro("Server Generation")?;

    let path: String = input("Where should we create your server?")
        .placeholder("./server")
        .validate(|input: &String| {
            if input.is_empty() {
                Err("Please enter a path.")
            } else if !input.starts_with("./") {
                Err("Please enter a relative path (ex. ./server)")
            } else {
                Ok(())
            }
        })
        .interact()?;

    let project_name: String = input("What should the name of this server be?")
    .placeholder(&path.replace("./", ""))
    .interact()?;

    let eula: bool = confirm("Do you accept the Mojang EULA?").initial_value(true).interact()?;

    let project = select("What type of server would you like to create?")
    .item("paper", "Paper", "Fork of Spigot")
    .item("velocity", "Velocity", "Performant Proxy")
    .interact()?;
    
    let server_manager = ServerManager::new(&project, &path).await?;

    if eula {
        server_manager.accept_eula()?;
    }

    let mut spinner = spinner();
    spinner.start("Fetching Versions...");
    let mut versions = server_manager.fetch_versions().await?;
    versions.sort_by(|a, b| b.cmp(a));
    spinner.stop("Fetched Versions.");

    
    cliclack::log::info(format!("Server Versions: {}", versions.join(", ")))?;
    let version: String = input("Choose a version")
    .placeholder(versions[2].clone().as_str())
    .validate(move |input: &String| {
        if !versions.contains(&input) {
            Err("Please enter a valid version.")
        } else {
            Ok(())
        }
    })
    .interact()?;

    spinner.start("Fetching Builds...");
    let mut builds = server_manager.fetch_builds(&version).await?;
    builds.sort_by(|a, b| b.cmp(a));
    spinner.stop("Fetched Builds.");

    
    cliclack::log::info(format!("Version Builds: {}", builds.join(", ")))?;
    let build: String = input("Choose a build")
    .placeholder(builds[0].clone().as_str())
    .validate(move |input: &String| {
        if !builds.contains(&input) {
            Err("Please enter a valid version.")
        } else {
            Ok(())
        }
    })
    .interact()?;

    let mut spinner2  = Spinner::default();
    spinner2.start("Downloading Server...");
    let server_jar_path = format!("{}/server.jar", path);
    let server_jar  = PathBuf::from(&server_jar_path);
    let url = server_manager.construct_download_url(&version, &build);
    server_manager.download_server(&url, &server_jar).await?;
    spinner2.stop("Downloaded Server.");

    outro("Finished Generation (Run the server with rustique run <name>")?;

    let mut config =Config::read_from_file(Config::default_config_path())?;
   config.add_server(
        Server {
            name: project_name,
            path: PathBuf::from(path).canonicalize()?.to_str().unwrap().to_string(),
            server_type: project.to_string(),
            version: version,
        }
    );
   config.save_to_file(Config::default_config_path()).expect("Failed to save config!");

    Ok(())
}

