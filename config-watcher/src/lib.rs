pub struct ConfigWatcher {
    running: parking_lot::Mutex<bool>,
    watched_paths: parking_lot::Mutex<Vec<std::path::PathBuf>>,
    callback: Option<Box<dyn Fn(WatchedFile) + Send + Sync + 'static>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchedFile {
    RcXml,
    MenuXml,
    ThemeRc,
    Autostart,
}

impl WatchedFile {
    pub fn name(self) -> &'static str {
        match self {
            Self::RcXml => "rc.xml",
            Self::MenuXml => "menu.xml",
            Self::ThemeRc => "themerc",
            Self::Autostart => "autostart",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigChangeKind {
    Modified,
    Created,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    pub file: WatchedFile,
    pub path: std::path::PathBuf,
    pub kind: ConfigChangeKind,
    pub timestamp: std::time::Instant,
}

impl ConfigWatcher {
    pub fn new() -> Self {
        Self {
            running: parking_lot::Mutex::new(false),
            watched_paths: parking_lot::Mutex::new(Vec::new()),
            callback: None,
        }
    }

    pub fn register_path(&self, path: &std::path::Path, file: WatchedFile) {
        self.watched_paths.lock().push(path.to_path_buf());
        tracing::info!("Watching {:?} ({})", path, file.name());
    }

    pub fn set_callback<F>(&mut self, cb: F)
    where
        F: Fn(ConfigChangeEvent) + Send + Sync + 'static,
    {
        self.callback = Some(Box::new(move |f: WatchedFile| {
            cb(ConfigChangeEvent {
                file: f,
                path: std::path::PathBuf::new(),
                kind: ConfigChangeKind::Modified,
                timestamp: std::time::Instant::now(),
            })
        }));
    }

    pub fn start_watching(&self) -> Result<(), Box<dyn std::error::Error>> {
        let paths = self.watched_paths.lock().clone();
        if paths.is_empty() {
            return Ok(());
        }

        let running = parking_lot::Mutex::new(true);
        *self.running.lock() = true;

        std::thread::spawn(move || {
            let mut inotify = match inotify::Inotify::init() {
                Ok(i) => i,
                Err(_) => return,
            };
            let mut wds = Vec::new();
            for p in &paths {
                if let Ok(wd) = inotify.watches().add(
                    p,
                    inotify::WatchMask::MODIFY
                        | inotify::WatchMask::CLOSE_WRITE
                        | inotify::WatchMask::CREATE,
                ) {
                    wds.push((p.clone(), wd));
                }
            }
            let mut buffer = [0u8; 4096];
            loop {
                if !*running.lock() {
                    break;
                }
                match inotify.read_events_blocking(&mut buffer) {
                    Ok(events) => {
                        for ev in events {
                            if ev.mask.contains(inotify::EventMask::MODIFY)
                                || ev.mask.contains(inotify::EventMask::CLOSE_WRITE)
                            {
                                for (p, _) in &wds {
                                    tracing::info!("Config file changed: {:?}", p);
                                }
                            }
                        }
                    }
                    Err(_) => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        });
        Ok(())
    }

    pub fn stop(&self) {
        *self.running.lock() = false;
    }

    pub fn watch_all_configs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_home =
            dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("/etc/xdg"));
        let labwc_dir = config_home.join("labwc");

        let files = [
            (labwc_dir.join("rc.xml"), WatchedFile::RcXml),
            (labwc_dir.join("menu.xml"), WatchedFile::MenuXml),
            (labwc_dir.join("themerc"), WatchedFile::ThemeRc),
            (labwc_dir.join("autostart"), WatchedFile::Autostart),
        ];

        for (path, file) in &files {
            if path.exists() || path.parent().map(|p| p.exists()).unwrap_or(false) {
                self.register_path(path, *file);
            }
        }
        self.start_watching()
    }
}

impl Default for ConfigWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watched_file_names() {
        assert_eq!(WatchedFile::RcXml.name(), "rc.xml");
        assert_eq!(WatchedFile::MenuXml.name(), "menu.xml");
        assert_eq!(WatchedFile::ThemeRc.name(), "themerc");
    }

    #[test]
    fn test_watcher_new() {
        let w = ConfigWatcher::new();
        assert!(!*w.running.lock());
    }

    #[test]
    fn test_register_path() {
        let w = ConfigWatcher::new();
        w.register_path(std::path::Path::new("/tmp/test.xml"), WatchedFile::RcXml);
        assert_eq!(w.watched_paths.lock().len(), 1);
    }

    #[test]
    fn test_config_change_event() {
        let ev = ConfigChangeEvent {
            file: WatchedFile::RcXml,
            path: std::path::PathBuf::from("/tmp/rc.xml"),
            kind: ConfigChangeKind::Modified,
            timestamp: std::time::Instant::now(),
        };
        assert_eq!(ev.file, WatchedFile::RcXml);
    }
}
