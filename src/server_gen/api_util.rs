use colored::Colorize;
use serde_json::Value;


use std::{fs::{self, File}, io::{Write, BufReader, BufRead, self}, path::{PathBuf}, process::{Command, Stdio}, thread};

pub struct ServerManager {
    project_name: String,
    folder_name: PathBuf,
}

impl ServerManager {
    pub async fn new(project_name: &str, folder_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let folder_path = PathBuf::from(folder_name);
        Ok(Self {
            project_name: project_name.to_owned(),
            folder_name: folder_path,
        })
    }

   pub  async fn fetch_versions(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!("https://papermc.io/api/v2/projects/{}", self.project_name);
        let response = reqwest::get(&url).await?.text().await?;
        let data: Value = serde_json::from_str(&response)?;
        let versions = data["versions"].as_array().ok_or("No versions found")?
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect();

        Ok(versions)
    }

    pub async fn fetch_builds(&self, version: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!("https://papermc.io/api/v2/projects/{}/versions/{}", self.project_name, version);
        let response = reqwest::get(&url).await?.text().await?;
        let data: Value = serde_json::from_str(&response)?;
        let builds = data["builds"].as_array().ok_or("No builds found")?
            .iter()
            .map(|b| b.to_string())
            .collect();

        Ok(builds)
    }

   pub  fn construct_download_url(&self, version: &str, build: &str) -> String {
        format!(
            "https://papermc.io/api/v2/projects/{}/versions/{}/builds/{}/downloads/paper-{}-{}.jar",
            self.project_name, version, build, version, build
        )
    }

    pub async fn download_server(&self, url: &str, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let response = reqwest::get(url).await?;
        let mut file = File::create(file_path)?;
        let content = response.bytes().await?;
        file.write_all(&content)?;
        Ok(())
    }

   pub  fn accept_eula(&self) -> Result<(), Box<dyn std::error::Error>> {
        let eula_path = self.folder_name.join("eula.txt");
        let mut eula_file = File::create(eula_path)?;
        writeln!(eula_file, "eula=true")?;
        Ok(())
    }

    pub async fn run_server(&self, folder_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let mut child = Command::new("java")
            .current_dir(folder_path)
            .args(&["-jar", "server.jar", "nogui"])
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("Failed to open child stdin");
        let stdout = child.stdout.take().expect("Failed to open child stdout");

         let reader = BufReader::new(stdout);

        // Spawn a new thread to handle the output
        thread::spawn(move || {
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        println!("{} {}", "|MINECRAFT|".green(), line.white());
                    }
                    Err(e) => eprintln!("Error reading line: {}", e),
                }
            }
        });

        let rtdin = io::stdin();
        for line in rtdin.lock().lines() {
            let line = line.expect("Failed to read line");
            if !line.starts_with('#') {
                if let Err(e) = stdin.write_all(line.as_bytes()) {
                    eprintln!("Failed to write to child stdin: {}", e);
                    break; // Break out of the loop if an error occurs
                }
                if let Err(e) = stdin.write_all(b"\n") {
                    eprintln!("Failed to write newline to child stdin: {}", e);
                    break; // Break out of the loop if an error occurs
                }
            }
        }

        child.wait()?;
        Ok(())
    }

   pub async fn delete_server(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        fs::remove_dir_all(path)?;
        Ok(())
    }
}