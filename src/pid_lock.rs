use anyhow::{Context, Result};
use log::debug;
use std::fs;
use std::io::Write;

pub struct PidLock {
    path: String,
    file: Option<fs::File>,
}

impl PidLock {
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self {
            path: path.into(),
            file: None,
        }
    }

    pub fn acquire(&mut self) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(false)
            .open(&self.path)
            .context(format!("Could not open \"{}\"", &self.path))?;

        file.write_all(std::process::id().to_string().as_bytes())
            .context("Could not write PID to lock file")?;
        file.flush().context("Could not flush PID lock file")?;

        self.file = Some(file);
        debug!("Acquired lock file \"{}\"", &self.path);
        Ok(())
    }

    pub fn acquired(&self) -> bool {
        self.file.is_some()
    }
}

impl Drop for PidLock {
    fn drop(&mut self) {
        if self.acquired() {
            self.file = None;
        }
        fs::remove_file(&self.path)
            .unwrap_or_else(|_| panic!("Could not remove file \"{}\"", &self.path));

        debug!("Released lock file \"{}\"", &self.path);
    }
}
