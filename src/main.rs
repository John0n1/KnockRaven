use clap::{ArgAction, Parser};
use knockraven::{scan_sequences, scan_sequences_mixed, total_sequence_count, Protocol};
use std::net::ToSocketAddrs;

/// Knockraven – Multi‑protocol Port‑Knocking Discovery
///
/// This command‑line interface orchestrates the sequence generation and
/// scanning routines exposed by the Knockraven library.  Users specify the
/// target host, candidate ports, sequence length, and various timing and
/// concurrency parameters.  Knockraven will then attempt to brute force all
/// possible sequences and report those which successfully open the monitor
/// port.  The tool is intended for authorized security assessments and
/// research only.  Use it responsibly and only against systems you own or
/// have explicit permission to test.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target host name or IP address to knock
    #[arg(value_name = "HOST")]
    host: String,

    /// Candidate ports to include in sequences (space separated list)
    #[arg(value_name = "PORT", num_args = 1..)]
    ports: Vec<u16>,

    /// Length of the port knock sequence
    #[arg(short = 'l', long = "length", default_value_t = 3)]
    seq_len: usize,

    /// Transport protocol to use for knocks [tcp|udp|mixed].  In mixed mode
    /// Knockraven tests every combination of TCP and UDP knocks of the
    /// specified length.
    #[arg(short = 'p', long = "protocol", default_value = "tcp", value_parser = ["tcp", "udp", "mixed"])]
    protocol: String,

    /// Monitor port to check after each knock sequence (TCP only)
    #[arg(short = 'm', long = "monitor", default_value_t = 22)]
    monitor_port: u16,

    /// Delay between knocks in milliseconds
    #[arg(short = 'd', long = "delay", default_value_t = 200)]
    delay_ms: u64,

    /// Timeout for each connection attempt in milliseconds
    #[arg(short = 't', long = "timeout", default_value_t = 1000)]
    timeout_ms: u64,

    /// Maximum number of concurrent knock attempts
    #[arg(short = 'c', long = "concurrency", default_value_t = 10)]
    concurrency: usize,

    /// Print verbose output (may be specified multiple times)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    // Validate host by attempting DNS resolution.  This provides early feedback
    // if the host name is invalid.  We ignore the port because we only
    // determine connectivity at runtime.
    let _resolved: Vec<_> = match (args.host.as_str(), 0).to_socket_addrs() {
        Ok(addrs) => addrs.collect(),
        Err(e) => {
            eprintln!("error: unable to resolve host '{}': {}", args.host, e);
            std::process::exit(1);
        }
    };
    // Check that the sequence length is sensible.  Very large sequence
    // lengths will produce astronomical numbers of combinations and are
    // impractical to brute force.  We warn the user when the count exceeds
    // a threshold but still allow execution.
    let total = total_sequence_count(args.ports.len(), args.seq_len);
    if total > 100_000 {
        eprintln!("warning: {} sequences will be generated; this may take a very long time", total);
    }
    // Determine whether to perform single-protocol or mixed-protocol scanning.
    let proto_str = args.protocol.to_lowercase();
    match proto_str.as_str() {
        "tcp" | "udp" => {
            let protocol = if proto_str == "tcp" { Protocol::Tcp } else { Protocol::Udp };
            if args.verbose > 0 {
                println!(
                    "Knockraven scanning {} sequences against host {} using {:?} protocol",
                    total, args.host, protocol
                );
            }
            let matches = scan_sequences(
                args.host,
                args.ports.clone(),
                args.seq_len,
                protocol,
                args.delay_ms,
                args.monitor_port,
                args.timeout_ms,
                args.concurrency,
            )
            .await;
            if matches.is_empty() {
                println!("No matching sequences discovered.");
            } else {
                println!("Discovered {} matching sequence(s):", matches.len());
                for seq in matches {
                    println!(
                        "  {}",
                        seq.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
                    );
                }
            }
        }
        "mixed" => {
            // In mixed mode the number of combinations doubles for each knock in the sequence.
            let combinations = (args.ports.len() as u64).pow(args.seq_len as u32);
            let proto_combos = 1u64 << args.seq_len;
            let total_mixed = combinations * proto_combos;
            if args.verbose > 0 {
                println!(
                    "Knockraven scanning {} mixed sequences ({} port sequences × {} protocol assignments) against host {}",
                    total_mixed,
                    combinations,
                    proto_combos,
                    args.host
                );
            }
            let results = knockraven::scan_sequences_mixed(
                args.host,
                args.ports.clone(),
                args.seq_len,
                args.delay_ms,
                args.monitor_port,
                args.timeout_ms,
                args.concurrency,
            )
            .await;
            if results.is_empty() {
                println!("No matching sequences discovered.");
            } else {
                println!("Discovered {} matching sequence(s):", results.len());
                for (ports, protos) in results {
                    let seq_str = ports
                        .iter()
                        .zip(protos.iter())
                        .map(|(p, proto)| {
                            let proto_label = match proto {
                                Protocol::Tcp => "tcp",
                                Protocol::Udp => "udp",
                            };
                            format!("{}/{}", p, proto_label)
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    println!("  {}", seq_str);
                }
            }
        }
        _ => unreachable!(),
    }
}