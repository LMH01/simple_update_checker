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