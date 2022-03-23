mod config;
mod fifo;
mod pid_lock;
mod process;

use std::io::Write;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use structopt::StructOpt;

use crate::fifo::Fifo;
use crate::pid_lock::PidLock;
use crate::process::{Process, ProcessDesc};

#[derive(StructOpt)]
#[structopt(name = "brt", about = "A bar wrapper")]
struct Opt {
    /// Path to config file
    #[structopt(short, long, default_value = "~/.config/barrette/barrette.toml")]
    config: String,
    /// Set log level to DEBUG
    #[structopt(short, long)]
    debug: bool,
    /// Disable logging
    #[structopt(short, long)]
    quiet: bool,
    /// Name of the command to run
    #[structopt(name = "COMMAND")]
    command: String,
}

fn init_logging(debug: bool) {
    let level = if debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    env_logger::Builder::new()
        .filter(None, level)
        .write_style(env_logger::fmt::WriteStyle::Always)
        .format_module_path(false)
        .format_timestamp(None)
        .init();
}

fn extract_percentage(raw: &str, re: &Regex) -> Option<String> {
    re.captures(raw)
        // and_then ~= flatmap
        .and_then(|c| c.get(0).map(|c| c.as_str().to_string()))
}

fn ensure_proc_is_running(conf: &config::Config) -> Result<()> {
    let proc = Process::from_process_desc(conf.process());
    let fifo = Fifo::from_process(conf.process());
    proc.ensure_is_running(fifo)?;
    Ok(())
}

fn get_percentage_from_command(conf: &config::Config, name: &str) -> Result<u32> {
    let cmd = conf
        .get_command_by_name(name)
        .ok_or_else(|| anyhow!("No such command: \"{}\"", name))?;

    let re = cmd.regex()?;
    let output = Process::from_process_desc(cmd).run()?;

    let raw_percentage = extract_percentage(&output, &re)
        .ok_or_else(|| anyhow!("Could not extract percentage from command output"))?;
    raw_percentage
        .replace("%", "")
        .parse()
        .context("Could not extract u32 from percentage")
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    if !opt.quiet {
        init_logging(opt.debug);
    }
    let conf_path = shellexpand::full(&opt.config)?;
    let conf =
        config::Config::from_config_file(&conf_path).context("could not read config file")?;

    let mut lock = PidLock::new(match conf.process().lock_path() {
        Some(s) => &s,
        None => "/tmp/barrette.pid",
    });
    lock.acquire()?;

    ensure_proc_is_running(&conf).with_context(|| {
        anyhow!(
            "Could not ensure process {} is running",
            &conf.process().command()
        )
    })?;

    let percentage = get_percentage_from_command(&conf, &opt.command)
        .with_context(|| "Could not extract percentage from command")?;
    log::debug!("Extracted percentage is {}", percentage);

    let mut fifo_file = Fifo::from_process(conf.process())
        .open_w()
        .context("could open FIFO for writing")?;

    fifo_file
        .write_all(format!("{}\n", percentage).as_bytes())
        .context("Could not write into FIFO")?;

    fifo_file.flush().context("Could not flush FIFO")
}
