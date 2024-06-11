use anyhow::{anyhow, Context, Result};
use sysinfo::{ProcessRefreshKind, RefreshKind};

use log::info;

use crate::fifo;

pub trait ProcessDesc {
    fn command(&self) -> &str;
    fn args(&self) -> Option<&Vec<String>>;
}

pub struct Process {
    name: String,
    args: Vec<String>,
}

impl Process {
    pub fn new<S1: AsRef<str>, S2: AsRef<str>>(name: S1, args: &[S2]) -> Self {
        fn to_str<S: AsRef<str>>(s: S) -> String {
            String::from(s.as_ref())
        }
        Self {
            name: to_str(name),
            args: args.iter().map(to_str).collect(),
        }
    }

    pub fn from_process_desc<T: ProcessDesc>(desc: &T) -> Self {
        Self::new(
            desc.command(),
            if let Some(arg_slice) = desc.args() {
                arg_slice
            } else {
                &[]
            },
        )
    }

    pub fn is_running(&self) -> bool {
        sysinfo::System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
        )
        .processes_by_exact_name(&self.name)
        .any(|p| p.name() == self.name)
    }

    pub fn spawn(&self, fifo_path: &str) -> Result<()> {
        // must be opened in RW, blocks otherwise
        let stdin = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(fifo_path)?;
        std::process::Command::new(&self.name)
            .args(&self.args)
            .stdin(stdin)
            .spawn()?;
        Ok(())
    }

    pub fn run(&self) -> Result<String> {
        let output = std::process::Command::new(&self.name)
            .args(&self.args)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(anyhow!(
                "Command was not successful: \"{}\"",
                String::from_utf8(output.stderr).context("stderr is not valid UTF-8")?
            ))
        }
    }

    pub fn ensure_is_running(&self, fifo: fifo::Fifo) -> Result<()> {
        if self.is_running() {
            info!("Process \"{}\" is already running", self.name);
        } else {
            info!("Process \"{}\" isn't running, starting...", self.name);
            fifo.ensure_exists()
                .context("could not ensure that fifo exists")?;
            self.spawn(fifo.path())
                .context(format!("could not spawn process \"{}\"", self.name))?;
            info!("Started process \"{}\"", self.name);
        }
        Ok(())
    }
}
