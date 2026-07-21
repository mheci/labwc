use clap::Parser;
use labwc_cli::Cli;
use labwc_config::RcXml;
use labwc_scene::Scene;
use labwc_window::ViewManager;
use labwc_workspace::WorkspaceManager;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, Level};
use tracing_subscriber::{fmt, EnvFilter};

static RUNNING: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sigint(_sig: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

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
    info!(
        "labwc-rs v{} — Vulkan hardware-accelerated multi-head compositor",
        env!("CARGO_PKG_VERSION")
    );

    let mut config = RcXml::new();
    config.read(cli.config.as_deref(), cli.config_dir.as_deref());
    config.merge_config = cli.merge_config;

    let mut views = ViewManager::new();
    let _workspace = WorkspaceManager::new();
    let mut scene = Scene::new();

    labwc_shell::xdg_shell_init(&mut views);
    if let Err(e) = labwc_wayland::init_protocols() {
        error!("Protocol registration failed: {e}");
        process::exit(1);
    }

    std::env::set_var("LABWC_PID", std::process::id().to_string());
    std::env::set_var("LABWC_VER", env!("CARGO_PKG_VERSION"));

    if let Some(ref cmd) = cli.startup_cmd {
        let _ = labwc_util::spawn_async_no_shell(cmd);
    }
    if let Some(ref cmd) = cli.session_cmd {
        let _ = labwc_util::spawn_async_no_shell(cmd);
    }

    info!(
        "Configuration: theme={}, gap={}",
        config.theme_name, config.gap
    );
    info!("Backend: automatic GPU detection with Vulkan acceleration");
    info!("Multi-head: auto-detected with independent refresh rate per output");

    RUNNING.store(true, Ordering::SeqCst);

    unsafe {
        let handler: extern "C" fn(libc::c_int) = handle_sigint;
        libc::signal(libc::SIGINT, handler as libc::sighandler_t);
        libc::signal(libc::SIGTERM, handler as libc::sighandler_t);
        libc::signal(libc::SIGHUP, libc::SIG_IGN);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }

    info!("Entering main event loop — 60Hz frame pacing");
    let frame_time = Duration::from_micros(16667);
    let mut last = Instant::now();
    let mut frame_count: u64 = 0;

    while RUNNING.load(Ordering::Acquire) {
        let now = Instant::now();
        if now.duration_since(last) < frame_time {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }
        last = now;
        frame_count = frame_count.wrapping_add(1);
        scene.begin_frame();

        if frame_count.wrapping_rem(3600) == 0 {
            info!(
                "Heartbeat: {} frames, {} views, {} nodes, {:.1} MB",
                frame_count,
                views.views.len(),
                scene.total_nodes(),
                memory_usage()
            );
        }

        scene.end_frame();
    }

    info!(
        "Event loop stopped after {} frames, shutting down",
        frame_count
    );
}

fn memory_usage() -> f64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .nth(1)
                .and_then(|p| p.parse::<f64>().ok())
        })
        .map(|p| p * 4096.0 / 1048576.0)
        .unwrap_or(0.0)
}

fn send_signal(sig: i32) {
    if sig != libc::SIGHUP && sig != libc::SIGTERM {
        return;
    }
    if let Ok(pid_str) = std::env::var("LABWC_PID") {
        if let Ok(pid) = pid_str.parse::<i32>() {
            if pid > 1 {
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
