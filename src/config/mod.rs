use colored::Colorize;
use serde::{Serialize, Deserialize};
use tokio::fs::File;
use std::{fs::{self,}, io, path::{Path, PathBuf}};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Server {
    pub name: String,
    pub path: String,
    pub server_type: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub servers: Vec<Server>,
}

impl Config {
    pub fn default_config_path() -> PathBuf {
        // Hi, I'm a comment. I'm here to tell you that this is a comment.
        if cfg!(target_os = "windows") {
            dirs::config_dir().unwrap_or_else(|| PathBuf::from("C:/"))
                .join("rustique")
                .join("config.toml")
        } else if cfg!(target_os = "macos") {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/Users/"))
                .join(".config")
                .join("rustique")
                .join("config.toml")
        } else {
            dirs::config_dir().unwrap_or_else(|| PathBuf::from("/etc/"))
                .join("rustique")
                .join("config.toml")
        }
    }

    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Config, io::Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if !path.exists() {
            let config = Config { servers: Vec::new() };
            let toml_string = toml::to_string(&config)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
            fs::write(path, toml_string)?;
        }

        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Ok(config)
    }

    pub fn add_server(&mut self, server: Server) {
        self.servers.push(server);
    }

    pub fn get_server(&self, name: &str) -> Option<&Server> {
        self.servers.iter().find(|s| s.name == name)
    }

    pub fn remove_server(&mut self, server: &Server) {
        self.servers.retain(|s| s.name != server.name);
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let toml_string = toml::to_string(self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        fs::write(path, toml_string)
    }

    pub fn display_servers() {
        let path = Config::default_config_path();
            match Config::read_from_file(path) {
                Ok(config) => {
                    println!("{}", "╔══ Servers Configuration ══╗".underline().bold());
                    for (i, server) in config.servers.iter().enumerate() {
                        println!("{} {}", "║".bold(), format!("Server #{}:", i + 1).bold().green());
                        println!("{} → Name: {}", "║".bold(), server.name.cyan());
                        println!("{} → Path: {}", "║".bold(), server.path.yellow());
                        println!("{} → Type: {}", "║".bold(), server.server_type.magenta());
                        println!("{} → Version: {}", "║".bold(), server.version.blue());
                        if i < config.servers.len() - 1 {
                            println!("{}─────", "║".bold());
                        }
                    }
                    println!("{}", "╚════════════════════════╝".underline().bold());
                }
                Err(e) => {
                    println!("Error reading config file: {:?}", e);
                }
            }
        }
}
