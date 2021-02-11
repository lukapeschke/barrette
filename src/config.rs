use crate::process;
use anyhow::Result;
use serde_derive::Deserialize;

use regex::Regex;

#[derive(Deserialize, Debug)]
pub struct Process {
    command: String,
    args: Option<Vec<String>>,
    fifo_path: Option<String>,
    fifo_mode: Option<u32>,
}

impl process::ProcessDesc for Process {
    fn command(&self) -> &str {
        &self.command
    }

    fn args(&self) -> Option<&Vec<String>> {
        self.args.as_ref()
    }
}

impl Process {
    pub fn fifo_path(&self) -> &Option<String> {
        &self.fifo_path
    }

    pub fn fifo_mode(&self) -> &Option<u32> {
        &self.fifo_mode
    }
}

#[derive(Deserialize, Debug)]
pub struct Command {
    name: String,
    command: String,
    args: Option<Vec<String>>,
    regex: Option<String>,
}

impl process::ProcessDesc for Command {
    fn command(&self) -> &str {
        &self.command
    }

    fn args(&self) -> Option<&Vec<String>> {
        self.args.as_ref()
    }
}

impl Command {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn regex(&self) -> Result<Regex> {
        Ok(Regex::new(match &self.regex {
            Some(s) => s,
            None => "([0-9]+)%",
        })?)
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    process: Process,
    commands: Vec<Command>,
}

impl Config {
    pub fn from_config_file(path: &str) -> Result<Config> {
        // The "?"operator implicit does a ".into()", so this is
        // equivalent to:
        //
        //   match toml::from_str(&std::fs::read_to_string(path)?) {
        //       Ok(c) => Ok(c),
        //       Err(e) => Err(e.into()),
        //   }
        //
        // Directly returning
        //   toml::from_str(&std::fs::read_to_string(path)?)
        //
        // would not work since toml's and anyhow's error types aren't
        // the same

        Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
    }

    pub fn process(&self) -> &Process {
        &self.process
    }

    pub fn get_command_by_name(&self, name: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.name() == name)
    }
}
