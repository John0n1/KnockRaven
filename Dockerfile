# syntax=docker/dockerfile:1
# Dockerfile for building and running Knockraven

## Stage 1: Build the binary in a clean Rust environment
FROM rust:1.73-slim as builder

WORKDIR /usr/src/knockraven

# Install required build tools.  The base rust image already includes
# rustc/cargo but not a C toolchain.
RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the project files and build
COPY . .
RUN cargo build --release

## Stage 2: Copy the binary into a minimal runtime image
FROM debian:stable-slim AS runtime

# Create non-root user for running the binary (optional)
RUN useradd -m knockraven

COPY --from=builder /usr/src/knockraven/target/release/knockraven /usr/local/bin/knockraven

USER knockraven
WORKDIR /home/knockraven

# Expose no ports by default; Knockraven is a CLI tool
ENTRYPOINT ["/usr/local/bin/knockraven"]

# Default command shows help
CMD ["--help"]