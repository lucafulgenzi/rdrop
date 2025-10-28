use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use clap::{Parser};
use twelf::{config, Layer};
use dirs::home_dir;

#[derive(Parser, Debug)]
#[command(
    name = "hdrop",
    version = "0.0.1",
    about = "Terminal dropdown utils for Hyprland (based on hyprctl)"
)]
struct Args {
    #[arg(short = 'c', long, default_value_t = default_config_path())]
    config: String,
}

#[config]
#[derive(Debug, Default, Serialize)]
struct Config {
    terminal: String,
    class: String,
    float: bool
}

#[derive(Deserialize)]
struct Workspace {
    id: i32,
    name: String,
}

/// Define struct for hyprctl client
#[derive(Deserialize)]
struct Client {
    class: String,
    workspace: Workspace,
}




// -- Configs functions --

fn default_config_path() -> String {
    home_dir()
        .map(|p| p.join(".config/rdrop/rdrop.yaml").to_string_lossy().into_owned())
        .unwrap_or_else(|| "/tmp/rdrop.yaml".to_string())
}


fn load_configs(path: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let path = path.into();
    let conf = Config::with_layers(&[
        Layer::Yaml(path)
    ])?;
    Ok(conf)
}

// -- Helper functions --

fn get_json_output(cmd: &str, args: &[&str]) -> io::Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

//
fn get_clients() -> Result<Vec<Client>, Box<dyn std::error::Error>> {
    let out = get_json_output("hyprctl", &["-j", "clients"])?;
    let clients: Vec<Client> = serde_json::from_str(&out)?;
    Ok(clients)
}


fn find_terminal(class: String) -> Option<Client> {
    let clients = match get_clients() {
        Ok(clients) => clients,
        Err(e) => {
            eprintln!("No clients found: {}", e);
            return None;
        }
    };

    clients.into_iter().find(|c| c.class == class)
}

fn main() {
    let args = Args::parse();
    let config: Config = match load_configs(PathBuf::from(args.config)) {
        Ok(config) => config,
        Err(e) => {
            std::process::exit(1);
        }
    };

    let terminal_client = find_terminal(config.class);

    if terminal_client.is_none() {
        // Create it
        println!("No terminal found");
    }else {
        // Show it / Move it
    }

}
