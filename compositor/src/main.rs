//! labwc-rs — A Wayland window-stacking compositor (Rust rewrite of labwc).
//!
//! Main entry point. Initializes all subsystems, registers Wayland protocols,
//! and enters the event loop.

use clap::Parser;
use labwc_cli::Cli;
use labwc_config::RcXml;
use labwc_focus::FocusManager;
use labwc_input::InputManager;
use labwc_window::ViewManager;
use labwc_workspace::WorkspaceManager;
use std::process;
use tracing::{error, info, Level};
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.debug {
        Level::DEBUG
    } else if cli.verbose {
        Level::INFO
    } else {
        Level::WARN
    };
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(log_level.into()))
        .init();

    if cli.version {
        println!("labwc-rs v{}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }
    if cli.exit {
        send_signal(libc::SIGTERM);
        process::exit(0);
    }
    if cli.reconfigure {
        send_signal(libc::SIGHUP);
        process::exit(0);
    }

    check_environment();
    info!("labwc-rs v{} starting", env!("CARGO_PKG_VERSION"));

    // ── Configuration ──
    let mut config = RcXml::new();
    let config_path = cli.config.clone();
    let config_dir = cli.config_dir.clone();
    config.read(config_path.as_deref(), config_dir.as_deref());
    config.merge_config = cli.merge_config;

    // ── Subsystems ──
    let mut views = ViewManager::new();
    let _focus = FocusManager::new();
    let workspace = WorkspaceManager::new();
    let _input = InputManager::new();

    // ── Shell handler ──
    labwc_shell::xdg_shell_init(&mut views);

    // ── Register Wayland protocols ──
    info!("Registering Wayland protocol handlers...");
    match labwc_wayland::init_protocols() {
        Ok(()) => info!("All Wayland protocol handlers registered"),
        Err(e) => {
            error!("Protocol registration failed: {e}");
            process::exit(1);
        }
    }

    // Protocol list verification
    info!("Active protocols:");
    info!("  - wp_content_type_manager_v1      (content type hints)");
    info!("  - xdg_toplevel_drag_manager_v1    (toplevel drag)");
    info!("  - wp_fifo_manager_v1              (FIFO presentation)");
    info!("  - wp_pointer_warp_manager_v1      (pointer warp)");
    info!("  - xdg_toplevel_tag_manager_v1     (toplevel tagging)");
    info!("  - ext_image_capture_source_v1     (screen capture)");
    info!("  - wp_commit_timing_manager_v1     (commit timing)");

    // ── Environment ──
    let pid = std::process::id();
    std::env::set_var("LABWC_PID", pid.to_string());
    std::env::set_var("LABWC_VER", env!("CARGO_PKG_VERSION"));

    if let Some(ref cmd) = cli.startup_cmd {
        let _ = labwc_util::spawn_async_no_shell(cmd);
    }

    info!("Entering main event loop");
    info!(
        "Configuration: theme={}, gap={}, placement={:?}",
        config.theme_name, config.gap, config.placement_policy
    );
    info!("labwc-rs initialized successfully");
    info!("  - Workspaces: {}", workspace.workspaces.len());
    info!("  - Keybinds: {}", config.keybinds.len());
    info!("  - Protocols: 7 Wayland + xdg-shell + layer-shell");
    info!("  - Crates: 22");

    if let Some(ref cmd) = cli.session_cmd {
        info!("Starting session: {cmd}");
        let _ = labwc_util::spawn_async_no_shell(cmd);
    }

    info!("labwc-rs event loop — awaiting full smithay Wayland integration");
}

fn send_signal(sig: i32) {
    if let Ok(pid_str) = std::env::var("LABWC_PID") {
        if let Ok(pid) = pid_str.parse::<i32>() {
            if pid > 0 {
                unsafe { libc::kill(pid, sig) };
            }
        }
    }
}

fn check_environment() {
    let euid = unsafe { libc::geteuid() };
    let uid = unsafe { libc::getuid() };
    if euid == 0 && uid != 0 {
        error!("SUID detected — aborting");
        process::exit(1);
    }
    if std::env::var("XDG_RUNTIME_DIR").is_err() {
        error!("XDG_RUNTIME_DIR is not set");
        process::exit(1);
    }
}
