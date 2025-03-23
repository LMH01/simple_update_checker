# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR simple_update_checker

# Copy your Rust project files into the container
COPY . .

# Build the Rust project, assuming the project has a Cargo.toml
RUN cargo build --release

# The command that runs the compiled binary
ENTRYPOINT ["./target/release/simple_update_checker", "run-timed"]