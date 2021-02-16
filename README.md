# barrette (`brt`)

``barrette`` is a user-friendly wrapper for [wob](https://github.com/francma/wob) /
[xob](https://github.com/florentc/xob) / overlay bars reading percentages from STDIN. It provides the `brt`
command.

I created it because I found these tools nice, but a bit cumbersome to use. The idea is simple:

1. Run a pre-defined command, such as `brt volume_up`

2. Get a nice overlay bar showing a percentage returned by the command

## Installation

### From a pre-built executable

Grab a release from the [releases](https://github.com/lukapeschke/barrette/releases) page.

### From source (requires a recent rust version to be installed)

```
cargo install --branch main --git https://github.com/lukapeschke/barrette
```

## Usage

For a complete CLI reference, run `brt --help`

The first step is to create config file. See [example-wob.toml](https://github.com/lukapeschke/barrette/blob/main/example-wob.toml) to get an example. I use this file with [wob](https://github.com/francma/wob)
on [sway](https://github.com/swaywm/sway) on Fedora 33.

## Configuration

The config is written in [TOML](https://toml.io). By default, `brt` will expect to find it in
`~/.config/barrette/barrette.toml`, but another path can specified through the `-c/--config` CLI flag.

It is composed of a **process** and an array of **commands**:

```toml
[process]
# required, name of the command to use
command = "wob"
# optional, array of arguments to pass to the command
args = ["--background-color", "#55000000", "-b 0", "--bar-color", "#aaffffff"]
# optional, path of the FIFO to create that will be used as the command's STDIN. Defaults to "/tmp/barrette_fifo"
fifo_path = "/tmp/barrette_fifo"
# optional, permissions of the FIFO in octal notation (prefixed with 0o)
fifo_mode = 0o600

[[commands]]
# required, name that will be passed to brt when running "brt COMMAND"
name = "volume_up"
# required, name of the command to use
command = "amixer"
# optional, array of arguments to pass to the command
args = ["sset", "Master", "5%+"]
# optional, regex that should be used to extract the percentage from the command's output. Defaults to "([0-9]+)%".
# The regex is compiled and matched using rust's regex library. It must include one unnamed capture group. Percentage
# symbols ('%') will be removed.
regex = "([0-9]+)%"

[[commands]]
name = "other_command"
command = "..."
```

Additional commands can be added in new `[[commands]]` sections.

## How it works

Admitting that we've run `brt volume_up` with the config above:

1. First, brt checks if the process is already running. If it is, it directly skips to step 4.
2. If not, it checks if the FIFO described in the config exists, and creates if if needed.
3. The process is started.
4. The command matching "volume_up" is run
5. The command's output is parsed and the percentage is extracted
6. The extracted percentage is written to the previously created FIFO, followed by a carriage return (`\n`)
