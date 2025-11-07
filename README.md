# Knockraven – Multi‑Protocol Port‑Knocking Discovery Tool

<p align="center">
  <img src="docs/logo.png" alt="Knockraven logo" width="128" />
</p>

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![License](https://img.shields.io/badge/license-MIT-blue)](#)
[![Platform](https://img.shields.io/badge/platform-Linux-orange)](#)
[![Language](https://img.shields.io/badge/language-Rust%20%2B%20C-blueviolet)](#)

## Overview

**Knockraven** is a research‑oriented command‑line utility that attempts to
discover services protected by port‑knocking schemes.  In a typical
configuration, a host will refuse all incoming connections until it receives a
specific sequence of connection attempts to closed ports; once the correct
sequence is seen, the firewall temporarily opens a hidden service port.
Port‑knocking offers a form of *security through obscurity* by concealing
services from casual scans【29475989647708†L112-L121】.  Adversaries have been observed
using port‑knocking to hide command‑and‑control listeners and persist on
compromised systems【804619713130562†L56-L64】.  Conversely, defenders and penetration
testers need tools to enumerate these sequences because a port protected by a
competently implemented knock is “nearly impossible to discover using active
probes such as those sent by Nmap”【688072229527641†L141-L145】.

Knockraven fills this gap.  It brute forces sequences of TCP or UDP connection
attempts, monitors for a subsequent service opening and reports any
combinations that work.  The engine supports:

* **Multi‑protocol knocking.** You can choose TCP or UDP knocks for the entire
  sequence.  Mixed mode is also supported, in which Knockraven tests every
  combination of TCP and UDP for each position in the sequence.
* **Configurable sequence length.** Real‑world port‑knocking schemes commonly
  use three‑ or four‑stage sequences【29475989647708†L112-L121】, but Knockraven lets you
  try arbitrary lengths.
* **Concurrent scanning.** An asynchronous backend dispatches multiple
  sequences in parallel to accelerate discovery.
* **Modular architecture.** The core sequence generator is written in C and
  exposed to Rust via a small FFI wrapper.  The scanning engine is built in
  Rust with Tokio for efficient concurrency.  Shell scripts and Makefiles
  simplify integration and packaging.

> **Important:** This tool is provided for educational and authorized
> security research.  Use it only against systems you own or have explicit
> permission to test.  Misuse could be illegal and unethical.

## Building

Knockraven requires a recent Rust toolchain (edition 2021) and a C compiler.
On Debian or Ubuntu based distributions you can install the prerequisites with:

```sh
sudo apt update && sudo apt install -y build-essential curl
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
```

Clone the repository and build the project using Cargo (the Rust package
manager).  If you are on a Debian‑based distribution you can instead
use the provided packaging metadata to build a proper `.deb` package via
`make deb` as described below.

```sh
git clone https://example.com/knockraven.git
cd knockraven
cargo build --release
```

To produce a Debian package you will need `dpkg-dev` and `debhelper`.
After installing these dependencies you can run:

```sh
make deb
```

This will build the Rust code, create a `knockraven_VERSION_ARCH.deb`
package under the project root using the metadata under the `debian/`
directory and leave it in the parent directory.  You can then install
it with `sudo dpkg -i knockraven_*.deb`.

The resulting binary will be located at `target/release/knockraven`.  You can
also build using the included `Makefile` which wraps `cargo`:

```sh
make
```

### Docker

A `Dockerfile` is provided for environments where a Rust toolchain is not
readily available.  Build and run Knockraven in an isolated container like so:

```sh
docker build -t knockraven .
docker run --rm -it --network host knockraven --help
```

### Dev Container

A VS Code [devcontainer](https://code.visualstudio.com/docs/devcontainers/overview)
configuration is included under `.devcontainer/`.  It sets up a complete
development environment with Rust, C toolchains and all project dependencies.
Open the folder in VS Code and use the *Reopen in Container* command to get
started.

## Usage

Run the `knockraven` binary with a target host and a list of candidate ports.
Specify the sequence length and other parameters via command‑line flags.  For
example, to test all three‑port combinations of the ports `1111`, `2222` and
`3333` against `192.0.2.10` using TCP knocks and monitoring SSH on port 22:

```sh
sudo ./knockraven 192.0.2.10 1111 2222 3333 \
    --length 3 \
    --protocol tcp \
    --monitor 22 \
    --delay 200 \
    --timeout 1000 \
    --concurrency 10
```

The tool will print a warning if the number of sequences is large.  Once
scanning completes, any successful sequences will be listed as comma‑separated
port numbers.  Increase the delay if the target’s port‑knocking daemon expects
longer gaps between knocks.  Use `--protocol udp` to send UDP datagrams
instead of TCP SYNs.

### Options

| Flag | Description |
|------|-------------|
| `HOST` | Hostname or IP address to target. |
| `PORT` | List of candidate ports (space separated). |
| `-l`, `--length` | Length of the knock sequence (default: 3). |
| `-p`, `--protocol` | Transport protocol (`tcp`, `udp` or `mixed`) for knocks.  In mixed mode Knockraven enumerates every combination of TCP/UDP for the sequence. |
| `-m`, `--monitor` | Port to check after each sequence (default: 22). |
| `-d`, `--delay` | Delay between knocks in milliseconds (default: 200). |
| `-t`, `--timeout` | Timeout per connection attempt in milliseconds (default: 1000). |
| `-c`, `--concurrency` | Number of concurrent sequences to test (default: 10). |
| `-v`, `--verbose` | Increase verbosity; repeat for more detail. |

Run `./knockraven --help` to see the full usage information.

## How It Works

1. **Sequence generation:** Knockraven calls into a C engine which
   generates all possible sequences of a given length from the provided list
   of ports.  If you specify three ports and a sequence length of three,
   there are `3^3 = 27` combinations.  Each combination is fed back to
   Rust for processing.
2. **Knock execution:** For each candidate sequence, Knockraven sends a
   connection attempt to each port in order.  For TCP it initiates a
   non‑blocking connect and immediately drops the connection; for UDP it
   transmits an empty datagram.  A configurable delay separates knocks to
   account for simple port‑knocking daemons that expect gaps.
3. **Monitoring:** After finishing the knocks, Knockraven attempts to open
   the monitor port using a TCP connection.  If the connection is accepted,
   the tool assumes that the port‑knocking sequence succeeded and records
   the sequence.
4. **Concurrency:** The scanner schedules multiple sequences concurrently
   using Tokio’s asynchronous runtime.  A semaphore limits the number of
   parallel sequences to avoid overwhelming the network or the target host.

## Design Rationale

Port‑knocking was designed to hide services from port scanners and casual
attackers.  Nmap’s own documentation notes that a service protected by port
knocking is “nearly impossible to discover using active probes”【688072229527641†L141-L145】.
Knockraven takes advantage of this weakness: instead of guessing which
service might be listening on which port, it focuses on uncovering the
sequence that unlocks the service.  By iterating through all possible
combinations of candidate ports and transport protocols, it can reveal
otherwise invisible services.  This capability is useful for penetration
testers performing authorized assessments and for defenders auditing their
networks for misconfigured or forgotten port‑knocking setups.

The tool also acknowledges the caveats around port‑knocking:
implementations can be fragile and susceptible to replay or brute force
attacks【688072229527641†L136-L139】, they should not replace proper
authentication【29475989647708†L123-L127】, and port‑knocking itself can be used
maliciously【804619713130562†L56-L64】.  Knockraven is therefore intended for
controlled testing and not as an endorsement of port‑knocking as a
security measure.

## File Layout

```
knockraven/
├── Cargo.toml       – Rust crate manifest
├── build.rs         – Build script compiling the C engine
├── c_engine/        – Simple C library for generating sequences
│   ├── seqgen.c
│   └── seqgen.h
├── src/
│   ├── lib.rs       – FFI bindings and scanning logic
│   └── main.rs      – Command‑line interface
├── docs/            – Additional documentation
├── Dockerfile       – Containerized build/run environment
├── Makefile         – Convenience build targets
├── .devcontainer/   – VS Code devcontainer setup
├── LICENSE          – MIT license
└── README.md        – This document
```

## Legal Notice

Knockraven is released under the MIT license; see `LICENSE` for the full
text.  The authors make no warranty as to the suitability of this software
for any purpose.  You are responsible for ensuring that your use of this tool
complies with local laws and regulations.