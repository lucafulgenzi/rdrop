use clap::Parser;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use twelf::{config, Layer};

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

#[derive(Debug, Serialize, Deserialize)]
enum TermPosition {
    T, // Top
    R, // Right
    B, // Bottom
    L, // Left
}

#[config]
#[derive(Debug, Serialize)]
struct Config {
    terminal: String,
    class: String,
    width: i32,
    height: i32,
    gap: i32,
    position: TermPosition,
}


#[derive(Deserialize)]
struct Workspace {
    id: i32,
    name: String,
}

#[derive(Deserialize)]
struct Monitor {
    width: i32,
    height: i32,
    focused: bool,
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
        .map(|p| {
            p.join(".config/rdrop/rdrop.yaml")
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|| "/tmp/rdrop.yaml".to_string())
}

fn load_configs(path: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let path = path.into();
    let conf = Config::with_layers(&[Layer::Yaml(path)])?;
    Ok(conf)
}

/// -- Helper functions --

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

fn dispatch_hyrpctl_command(args: &[&str]) -> io::Result<String> {
    let output = Command::new("hyprctl")
        .arg("dispatch")
        .arg("--")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn get_clients() -> Result<Vec<Client>, Box<dyn std::error::Error>> {
    let out = get_json_output("hyprctl", &["-j", "clients"])?;
    let clients: Vec<Client> = serde_json::from_str(&out)?;
    Ok(clients)
}

fn get_active_workspace() -> Result<Workspace, Box<dyn std::error::Error>> {
    let out = get_json_output("hyprctl", &["-j", "activeworkspace"])?;
    let ws: Workspace = serde_json::from_str(&out)?;
    Ok(ws)
}

fn get_monitors() -> Result<Vec<Monitor>, Box<dyn std::error::Error>> {
    let out = get_json_output("hyprctl", &["-j", "monitors"])?;
    let monitors: Vec<Monitor> = serde_json::from_str(&out)?;
    Ok(monitors)
}

fn find_terminal(class: &String) -> Option<Client> {
    let clients = match get_clients() {
        Ok(clients) => clients,
        Err(e) => {
            eprintln!("No clients found: {}", e);
            return None;
        }
    };

    clients.into_iter().find(|c| c.class.eq(class))
}

/// Get focused monitor from hyprctl
fn find_active_monitor() -> Option<Monitor> {
    let monitors = match get_monitors() {
        Ok(monitors) => monitors,
        Err(e) => {
            eprintln!("No monitor found: {}", e);
            return None;
        }
    };
    monitors.into_iter().find(|m| m.focused)
}

// TODO: add validation for width and height min 0, max 100
fn calc_terminal_percentage_size(width: i32, height: i32) -> (i32, i32) {
    let active_monitor = find_active_monitor();
    if active_monitor.is_none() {
        println!("No monitor found");
        return (-1, -1);
    }

    let monitor = active_monitor.unwrap();

    let terminal_width = (monitor.width * width) / 100;
    let terminal_height = (monitor.height * height) / 100;

    (terminal_width, terminal_height)
}



fn parse_commands(config: &Config, create: bool, terminal_workspace: i32) {
    if create {
        let mut command: Vec<&str> = ["exec", "[workspace special:rdrop silent;"].to_vec();
        command.push("float;");
        command.push("]");
        command.push(config.terminal.as_str());
        let class = format!("--class {}", config.class);
        command.push(class.as_str());
        let res = dispatch_hyrpctl_command(&*command).expect("TODO: panic message");
        println!("{}", res)
    } else {
        let mut command: Vec<String> = ["movetoworkspacesilent".to_string()].to_vec();

        let ws = get_active_workspace().expect("No active workspace found");
        if ws.id != terminal_workspace {
            println!("Moved into {}", ws.id);
            let move_to = format!("{},", ws.name);
            command.push(move_to);
        } else {
            println!("Put in special workspace");
            command.push("special:rdrop,".to_string())
        }

        let class = format!("class:{}", config.class);
        command.push(class);

        let command_refs: Vec<&str> = command.iter().map(|s| s.as_str()).collect();
        let res = dispatch_hyrpctl_command(&command_refs).expect("TODO: panic message");
        println!("{}", res);

        let (width, height) = calc_terminal_percentage_size(config.width, config.height);

        let mut resize_command: Vec<String> = [
            "resizewindowpixel".to_string(),
            "exact".to_string(),
            width.to_string(),
            height.to_string(),
            ",".to_string(),
        ]
        .to_vec();
        let class2 = format!("class:{}", config.class);
        resize_command.push(class2);
        let resize_command_refs: Vec<&str> = resize_command.iter().map(|s| s.as_str()).collect();
        dispatch_hyrpctl_command(&resize_command_refs).expect("TODO: panic message");



        let active_monitor = find_active_monitor();
        let monitor = active_monitor.unwrap();
        let (x_pos, y_pos) = match config.position {
            TermPosition::T => {
                let x = (monitor.width - width) / 2;
                let y = config.gap;
                (x, y)
            },
            TermPosition::R => {
                let x = config.gap;
                let y = (monitor.height - height) / 2;
                (x, y)
            },
            TermPosition::B => {
                let x = (monitor.width - width) / 2;
                let y = monitor.height - ( height + config.gap);
                (x, y)
            },
            TermPosition::L => {
                let x = monitor.width - (width + config.gap);
                let y = (monitor.height - height) / 2;
                (x, y)
            }
        };

        let mut move_command: Vec<String> = [
            "movewindowpixel".to_string(),
            "exact".to_string(),
            x_pos.to_string(),
            y_pos.to_string(),
            ",".to_string(),
        ]
        .to_vec();
        let class2 = format!("class:{}", config.class);
        move_command.push(class2);
        let move_command_refs: Vec<&str> = move_command.iter().map(|s| s.as_str()).collect();
        dispatch_hyrpctl_command(&move_command_refs).expect("TODO: panic message");
    }
}

fn main() {
    let args = Args::parse();
    let config: Config = match load_configs(PathBuf::from(args.config)) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    let terminal_client = find_terminal(&config.class);

    if terminal_client.is_none() {
        parse_commands(&config, true, 0);
    } else {
        parse_commands(&config, false, terminal_client.unwrap().workspace.id);
    }
}
