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
Using config file found at /home/louis/.config/simple_update_checker/config.toml
Not using db_path setting found in config file (/home/louis/.local/simple_update_checker/programs.db) as --db-path is set (programs.db)
Using database file: programs.db
Checking 2 programs for updates...
alpha_tui: update found v1.7.0 -> v1.8.0
simple_graph_algorithms: update found v0.1.0 -> v1.0.0

Summary of programs that have updates available:

+-------------------------+-----------------+----------------+----------+
| name                    | current_version | latest_version | provider |
+-------------------------+-----------------+----------------+----------+
| alpha_tui               | v1.7.0          | v1.8.0         | github   |
+-------------------------+-----------------+----------------+----------+
| simple_graph_algorithms | v0.1.0          | v1.0.0         | github   |
+-------------------------+-----------------+----------------+----------+
```

### Update current_version when program has been updated

```
./simple_update_checker update -n <NAME>
```

This updates the `current_version` to the `latest_version` that is stored in the database. Does not check if a newer version is available.

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

### Timed mode

```
./simple_update_checker run-timed -n <NTFY_TOPIC> -c <CHECK_INTERVAL>
```

In timed mode the update check will be performed every `<CHECK_INTERVAL>` seconds. When an update is found a notification is send to [ntfy.sh](http://ntfy.sh) under the topic `<NTFY_TOPIC>`.

This is the function that is run when using the docker container.

See [docker section](#docker) on how to setup the program using a docker container.

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

### Github API rate limiting

If you are not authenticated against the Github API the rate limit is 60 requests per hour (which should be enough). To increase the rate limit to 5000 requests per hour create a personal access token [here](https://github.com/settings/personal-access-tokens) and provide it to the program with `--github-access-token <GITHUB_ACCESS_TOKEN>`.

## Compile from source

To compile the program from source the rust toolchain is needed (install via [rustup](https://rustup.rs/)). Once installed you can run the program by typing `cargo run`. To submit arguments you can use `--`, for example `cargo run -- -h` will print help.

## Using nix

This Repository provides a flake. If you have flakes enabled you can use

```
nix shell github:lmh01/simple_update_checker
```

to start a shell in which `simple_update_checker` is installed.

Warning: the build is currently broken, flake needs fixing (pkg-config is not found).

## Docker

To setup the program using a docker container follow these steps:

1. Clone the repository
2. Copy `docker-compose-template.yml` to `docker-compose.yml` and change the value for `<NTFY_TOPIC>`. See [ntfy.sh](https://ntfy.sh) on how to setup the app.
3. Create a new folder named data
4. Initialize the database and add programs that should be watched for updates using the following command: `cargo run --release -- -d data/programs.db add-program -n <NAME> github -r <GITHUB_REPOSITORY>`
5. Start the docker container using `docker compose up -d`

If you would like to add more programs stop the docker container with `docker compose down` and use the above command to add more programs. You can also use all other commands of the tool with this db.