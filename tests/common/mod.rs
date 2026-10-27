#![allow(dead_code)]

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(prefix: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let name = format!("rmlib_{}_{}_{}_{}", prefix, std::process::id(), id, nanos);
        let path = std::env::temp_dir().join(name);
        fs::create_dir_all(&path).expect("failed to create temp dir");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn join(&self, p: &str) -> PathBuf {
        self.path.join(p)
    }

    pub fn write(&self, rel: &str, content: &[u8]) -> PathBuf {
        let full = self.path.join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("failed to create parent");
        }
        fs::write(&full, content).expect("failed to write file");
        full
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

pub struct HomeGuard {
    _guard: MutexGuard<'static, ()>,
    saved_home: Option<OsString>,
    #[cfg(windows)]
    saved_userprofile: Option<OsString>,
}

impl HomeGuard {
    pub fn new(home: &Path) -> Self {
        let guard = env_lock();
        let saved_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", home);
        }
        #[cfg(windows)]
        let saved_userprofile = {
            let saved = std::env::var_os("USERPROFILE");
            unsafe {
                std::env::set_var("USERPROFILE", home);
            }
            saved
        };
        Self {
            _guard: guard,
            saved_home,
            #[cfg(windows)]
            saved_userprofile,
        }
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.saved_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
        }
        #[cfg(windows)]
        unsafe {
            match &self.saved_userprofile {
                Some(v) => std::env::set_var("USERPROFILE", v),
                None => std::env::remove_var("USERPROFILE"),
            }
        }
    }
}

pub fn write_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent");
    }
    fs::write(path, content).expect("failed to write file");
}
