use clap::Parser;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use twelf::reexports::log;
use twelf::{config, Layer};

#[derive(Parser, Debug)]
#[command(
    name = "rdrop",
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
struct TerminalSize {
    width: i32,
    height: i32,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            width: 600,
            height: 400,
        }
    }
}

#[derive(Deserialize, PartialEq)]
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

#[derive(Deserialize)]
struct Client {
    class: String,
    workspace: Workspace,
}

// --- Configs functions ---

fn default_config_path() -> String {
    home_dir()
        .map(|p| {
            p.join(".config/rdrop/rdrop.yaml")
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|| "/tmp/rdrop.yaml".to_string())
}

fn load_configs(path: PathBuf) -> Result<Config, Box<dyn Error>> {
    let path = path.into();
    let conf = Config::with_layers(&[Layer::Yaml(path)])?;
    Ok(conf)
}

/// --- Helper functions for hyprctl commands ---

/// Exec external command and return the output in JSON
fn get_json_output(cmd: &str, args: &[&str]) -> io::Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to exec command: {} {}", cmd, err),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Exec hyprctl dispatch with args
fn dispatch_hyrpctl_command(args: &[&str]) -> io::Result<()> {
    let output = Command::new("hyprctl")
        .arg("dispatch")
        .arg("--")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to dispatch hyprctl command (args: {:?}): {}",
                args, err
            ),
        ));
    }
    Ok(())
}

// --- Helper functions for get hyprclt env ---
fn get_clients() -> Result<Vec<Client>, Box<dyn Error>> {
    let out = get_json_output("hyprctl", &["-j", "clients"])?;
    let clients: Vec<Client> = serde_json::from_str(&out)?;
    Ok(clients)
}

fn get_active_workspace() -> Result<Workspace, Box<dyn Error>> {
    let out = get_json_output("hyprctl", &["-j", "activeworkspace"])?;
    let ws: Workspace = serde_json::from_str(&out)?;
    Ok(ws)
}

fn get_monitors() -> Result<Vec<Monitor>, Box<dyn Error>> {
    let out = get_json_output("hyprctl", &["-j", "monitors"])?;
    let monitors: Vec<Monitor> = serde_json::from_str(&out)?;
    Ok(monitors)
}

fn find_terminal(class: &String) -> Result<Option<Client>, Box<dyn Error>> {
    let clients = get_clients()?;
    Ok(clients.into_iter().find(|c| c.class.eq(class)))
}

/// Get focused monitor from hyprctl
fn find_active_monitor() -> Result<Monitor, Box<dyn Error>> {
    let monitors = get_monitors()?;

    monitors
        .into_iter()
        .find(|m| m.focused)
        .ok_or_else(|| "No focused monitor found".into())
}

// --- Calcs logics and dispatch ---

/// Calculate TerminalSize from monitor sizes and configs percentages
fn calc_terminal_size_from_percentage(
    width_percent: i32,
    height_percent: i32,
    monitor: &Monitor,
) -> TerminalSize {
    let terminal_width = (monitor.width * width_percent) / 100;
    let terminal_height = (monitor.height * height_percent) / 100;
    TerminalSize {
        width: terminal_width,
        height: terminal_height,
    }
}

fn dispatch_terminal_positioning(
    class: &str,
    monitor: &Monitor,
    position: &TermPosition,
    gap: i32,
    terminal_size: &TerminalSize,
) -> Result<(), Box<dyn Error>> {
    let (x_calc, y_calc) = match position {
        TermPosition::T => {
            let x = (monitor.width - terminal_size.width) / 2;
            let y = gap;
            (x, y)
        }
        TermPosition::R => {
            let x = monitor.width - (terminal_size.width + gap);
            let y = (monitor.height - terminal_size.height) / 2;
            (x, y)
        }
        TermPosition::B => {
            let x = (monitor.width - terminal_size.width) / 2;
            let y = monitor.height - (terminal_size.height + gap);
            (x, y)
        }
        TermPosition::L => {
            let x = gap;
            let y = (monitor.height - terminal_size.height) / 2;
            (x, y)
        }
    };

    let x_pos = x_calc.to_string();
    let y_pos = y_calc.to_string();
    let class_arg = format!("class:{}", class);

    let command_args: Vec<&str> = vec![
        "movewindowpixel",
        "exact",
        x_pos.as_str(),
        y_pos.as_str(),
        ",",
        &class_arg,
    ];

    dispatch_hyrpctl_command(&command_args)?;
    Ok(())
}

fn dispatch_terminal_init(terminal: &str, class: &str) -> Result<(), Box<dyn Error>> {
    let command_args: Vec<&str> = vec![
        "exec",
        "[workspace special:rdrop silent;float;]",
        terminal,
        "--class",
        class,
    ];
    dispatch_hyrpctl_command(&command_args)?;
    Ok(())
}

fn dispatch_terminal_move(class: &str, terminal_workspace: i32) -> Result<(), Box<dyn Error>> {
    let class_arg = format!("class:{}", class);

    let ws = get_active_workspace()?;

    let destination_arg = if ws.id != terminal_workspace {
        format!("{},", ws.name)
    } else {
        String::from("special:rdrop,")
    };

    let command_args: Vec<&str> = vec!["movetoworkspacesilent", &destination_arg, &class_arg];
    dispatch_hyrpctl_command(&command_args)?;

    if ws.id != terminal_workspace {
        dispatch_terminal_focus(&class)?;
    }


    Ok(())
}

fn dispatch_terminal_resize(
    class: &str,
    terminal_size: &TerminalSize,
) -> Result<(), Box<dyn Error>> {
    let class_arg = format!("class:{}", class);
    let width_str = terminal_size.width.to_string();
    let height_str = terminal_size.height.to_string();

    let command_args: Vec<&str> = vec![
        "resizewindowpixel",
        "exact",
        &width_str,
        &height_str,
        ",",
        &class_arg,
    ];

    dispatch_hyrpctl_command(&command_args)?;
    Ok(())
}

fn dispatch_terminal_focus(class: &str) -> Result<(), Box<dyn Error>> {
    let class_arg = format!("class:{}", class);

    let command_args: Vec<&str> = vec!["focuswindow", &class_arg];

    dispatch_hyrpctl_command(&command_args)?;
    Ok(())
}

fn dispatch_terminal_pin(class: &str) -> Result<(), Box<dyn Error>> {
    let class_arg = format!("class:{}", class);
    let command_args: Vec<&str> = vec!["pin", &class_arg];
    dispatch_hyrpctl_command(&command_args)?;
    Ok(())
}

fn parse_commands(
    config: &Config,
    create: bool,
    terminal_workspace: Option<i32>,
) -> Result<(), Box<dyn Error>> {
    if create {
        log::info!("Terminal session creating...");
        dispatch_terminal_init(&config.terminal, &config.class)?;
    } else {
        log::info!("Move and resize terminal session...");

        let monitor = find_active_monitor()?;

        let terminal_size: TerminalSize =
            calc_terminal_size_from_percentage(config.width, config.height, &monitor);

        dispatch_terminal_resize(&config.class, &terminal_size)?;

        dispatch_terminal_positioning(
            &config.class,
            &monitor,
            &config.position,
            config.gap,
            &terminal_size,
        )?;

        dispatch_terminal_pin(&config.class)?;

        let tws = terminal_workspace.expect("Terminal workspace not found during move");
        dispatch_terminal_move(&config.class, tws)?;

    }
    Ok(())
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    let args = Args::parse();
    let config: Config = match load_configs(PathBuf::from(args.config)) {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load config {}", e);
            std::process::exit(1);
        }
    };

    let result = match find_terminal(&config.class) {
        Ok(Some(client)) => parse_commands(&config, false, Some(client.workspace.id)),
        Ok(None) => parse_commands(&config, true, None),
        Err(e) => {
            log::error!("Errore during terminal find: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        log::error!("Fatal error during command dispatch: {}", e);
        std::process::exit(1);
    }
}
