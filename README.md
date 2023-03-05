![SSSH DEMO GIF](https://github.com/pouriya/restcommander/releases/download/media/sssh-demo.gif)


# SSSH
Simply connect to your ssh server.

## Installation
Download the latest version:
* GNU/Linux:
    * Musl (Statically linked):       [download](https://github.com/pouriya/sssh/releases/download/latest/sssh-latest-x86_64-unknown-linux-musl-ubuntu-22.04)
    * GNU (Dynamic linking to glibc): [download](https://github.com/pouriya/sssh/releases/download/latest/sssh-latest-x86_64-unknown-linux-gnu-ubuntu-22.04)
    * Debian package (`.deb` file):  
        * Musl (Statically linked):       [download](https://github.com/pouriya/sssh/releases/download/latest/sssh-latest-x86_64-unknown-linux-musl-ubuntu-22.04.deb)
        * GNU (Dynamic linking to glibc): [download](https://github.com/pouriya/sssh/releases/download/latest/sssh-latest-x86_64-unknown-linux-gnu-ubuntu-22.04.deb)
* macOS:
    * v11: [download](https://github.com/pouriya/sssh/releases/download/latest/sssh-latest-x86_64-apple-darwin-macos-11)
    * v12: [download](https://github.com/pouriya/sssh/releases/download/latest/sssh-latest-x86_64-apple-darwin-macos-12)
* Windows:
    * MSVC: [download](https://github.com/pouriya/sssh/releases/download/latest/sssh-latest-x86_64-pc-windows-msvc-windows-2022.exe)
    * GNU:  [download](https://github.com/pouriya/sssh/releases/download/latest/sssh-latest-x86_64-pc-windows-gnu-windows-2022.exe)

## Usage
```shell
$ sssh --help
```
```text
Simply connect to your SSH server.

Usage: sssh [OPTIONS] [COMMAND]

Commands:
  select
          Select a server to connect from the terminal UI. (default)
  edit
          Edit configuration file to add/remove servers
  config
          Print current configuration file contents
  script
          Print current script file contents
  sample
          Samples for configuration and script
  help
          Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose
          Increase verbosity
          
          [env: SSSH_VERBOSE=]

  -q, --quiet
          Disable logging
          
          [env: SSSH_QUIET=]

  -c, --config-file <config-file>
          TOML Configuration file.
          
          For more information run `sssh sample config`
          
          [env: SSSH_CONFIG_FILE=]
          [default: /home/p/.config/sssh.toml]

  -s, --script-file <script-file>
          An executable file that will accept SSH info to connect to chosen server.
          
          For more information run `sssh sample script`
          
          [env: SSSH_SCRIPT_FILE=]
          [default: /home/p/.config/sssh.sh]

  -S, --skip-select
          Skip running final script
          
          [env: SSSH_SKIP_SELECT=]

  -e, --editor-command <editor-command>
          Editor command for editing configuration file
          
          [env: SSSH_EDITOR_COMMAND=]
          [default: nano]

  -E, --editor-argument <editor-argument>
          List of arguments passed to --editor-command
          
          [env: SSSH_EDITOR_ARGUMENTS=]
          [default: "-l {FILENAME}"]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```


# To contributors
I ❤️ PR from everyone and I appreciate your help but before opening a PR, Open an issue and describe your feature, bug, etc.

