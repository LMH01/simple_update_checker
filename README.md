[![Release](https://img.shields.io/github/v/release/lmh01/simple_update_checker)](https://github.com/lmh01/alpha_tui/releases)
[![License](https://img.shields.io/github/license/lmh01/simple_update_checker)](LICENSE)


# Simple Update Checker

This is a simple program that can check programs for updates. Currently only programs with releases on github are supported. Support for other sources might be added in the future.

## Basic usage

After building the program (release binaries might become available in the future) you can do the following:

### Add program to database:

Basic syntax:

```
./simple_update_checker add-program -n <NAME> github -r <GITHUB_REPOSITORY> 
```

#### Example
```
./simple_update_checker add-program -n alpha_tui github -r LMH01/alpha_tui 
```

Output:
```
Program alpha_tui successfully added to database!
```

When programs are added to the database, the currently latest version is stored in the database.

### Come back in the future and check for updates:

// TODO Update this to reflect changes with current_version
// I think that I can also remove the struct CheckedProgram as the information is now directly stored in database

```
./simple_update_checker check
```

Output might look like this:

```
Checking 2 programs for updates...
alpha_tui: update found v1.7.0 -> v1.8.0
simple_graph_algorithms: update found v0.1.0 -> v1.0.0

Summary of programs that have updates available:

+-------------------------+--------------+----------------+----------+
| name                    | last_version | latest_version | provider |
+-------------------------+--------------+----------------+----------+
| alpha_tui               | v1.7.0       | v1.8.0         | github   |
+-------------------------+--------------+----------------+----------+
| simple_graph_algorithms | v0.1.0       | v1.0.0         | github   |
+-------------------------+--------------+----------------+----------+
```

### View programs that are added to database

```
./simple_update_checker list-programs
```

Output:
```
The following programs are currently stored in the database:

+-------------------------+----------------+----------+
| name                    | latest_version | provider |
+-------------------------+----------------+----------+
| alpha_tui               | v1.8.0         | github   |
+-------------------------+----------------+----------+
| simple_graph_algorithms | v1.0.0         | github   |
+-------------------------+----------------+----------+

Note: the latest_version displayed here might not necessarily be the actual newest version. Use command 'check' to check all programs for updates.
```

### View help

```
./simple_update_checker help
```

## Config file

If you would like to specify the location for a default `programs.db` file, you can do so by creating a config file called `config.toml` and placing it in `~/.config/simple_update_checker/`.

Example file:

```
db_path = "/home/louis/.local/simple_update_checker/programs.db"
```

Note that the folder for the database file needs to exist already, if it does not exist, the program will fail to start. The database file is created automatically.

If the cli option `--db-path` is set, it overrides the setting from the config file.

If the config file does not exist and `--db-path` is not set, `programs.db` will be created in your current directory.

## Future plans

- [ ] Add a timed mode where the program periodically checks for updates and then sends a notification using ntfy.sh when updates are available (command already exists, but has no functionality)

## Compile from source

To compile the program from source the rust toolchain is needed (install via [rustup](https://rustup.rs/)). Once installed you can run the program by typing `cargo run`. To submit arguments you can use `--`, for example `cargo run -- -h` will print help.

## Using nix

This Repository provides a flake. If you have flakes enabled you can use

```
nix shell github:lmh01/simple_update_checker
```

to start a shell in which `simple_update_checker` is installed.

Warning: the build is currently broken, flake needs fixing (pkg-config is not found).