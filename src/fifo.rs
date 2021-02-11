use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::{fs, os::unix::prelude::PermissionsExt};

use anyhow::{anyhow, Context, Result};
use log::info;

use crate::config;

pub struct Fifo {
    path: String,
    mode: u32,
}

impl Fifo {
    pub fn new<S: Into<String>>(path: S, mode: u32) -> Self {
        Self {
            path: path.into(),
            mode,
        }
    }

    pub fn from_process(proc: &config::Process) -> Self {
        let path = match proc.fifo_path() {
            Some(s) => &s,
            None => "/tmp/barrette_fifo",
        };
        Self::new(path, proc.fifo_mode().unwrap_or(0o600))
    }

    fn is_fifo(&self, meta: &fs::Metadata) -> bool {
        meta.file_type().is_fifo()
    }

    fn is_rw(&self, meta: &fs::Metadata) -> bool {
        let user_mode = (meta.permissions().mode() % 0o1000) / 0o100;
        (6..=7).contains(&user_mode)
    }

    pub fn exists(&self) -> Result<bool> {
        let path = Path::new(&self.path);
        if !path.exists() {
            return Ok(false);
        }
        match path.metadata() {
            Ok(meta) => {
                if !self.is_fifo(&meta) {
                    Err(anyhow!("\"{}\" is not a FIFO", self.path))
                } else if !self.is_rw(&meta) {
                    Err(anyhow!("\"{}\" isn't RW", self.path))
                } else {
                    Ok(true)
                }
            }

            Err(e) => Err(anyhow!(
                "Could not retrieve metadata for \"{}\": {}",
                self.path,
                e
            )),
        }
    }

    pub fn create(&self) -> Result<()> {
        let cname = std::ffi::CString::new(self.path.as_str())?;
        match unsafe { libc::mkfifo(cname.as_ptr(), self.mode) } {
            0 => Ok(()),
            _ => Err(std::io::Error::last_os_error().into()),
        }
    }

    pub fn ensure_exists(&self) -> Result<()> {
        if self.exists()? {
            info!("FIFO \"{}\" already exists", self.path);
        } else {
            info!("FIFO \"{}\" does not exist, creating...", self.path);
            self.create()?;
            info!("Created FIFO \"{}\"", self.path);
        }
        Ok(())
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn open_w(&self) -> Result<fs::File> {
        std::fs::OpenOptions::new()
            .read(false)
            .write(true)
            .open(&self.path)
            .context("Could not open fifo for writing")
    }
}
