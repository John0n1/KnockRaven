# Knockraven Usage Guide

This document provides additional usage examples and clarifies some of the
options available with Knockraven.  You should already have built the
`knockraven` binary following the instructions in the top‑level
`README.md`.

## Basic Scan

Suppose you suspect that a host at `10.0.0.5` runs an SSH daemon protected
by a three‑port knock sequence composed of the ports `4242`, `5151` and
`6161`.  To test all possible three‑port combinations in TCP mode:

```bash
sudo ./knockraven 10.0.0.5 4242 5151 6161 \
    --length 3 \
    --protocol tcp \
    --monitor 22 \
    --delay 300 \
    --timeout 1000 \
    --concurrency 20
```

The tool will first print a warning if the number of sequences is large.  In
this case there are `3^3 = 27` sequences to test.  Each sequence will be
executed in its own asynchronous task, up to the concurrency limit (20 in
this example).  Adjust the delay to match the expected timing behaviour of
the target’s port‑knocking daemon.  Some daemons require a pause between
knocks; others may ignore knocks that are too close together.

If the correct sequence is found, Knockraven will report it at the end of
the scan, for example:

```
Discovered 1 matching sequence(s):
  5151,6161,4242
```

Note that the ordering matters.  In this example, `5151,6161,4242` was
accepted whereas `4242,5151,6161` would not trigger the service.

## UDP Mode

Some port‑knocking implementations expect UDP datagrams instead of TCP SYN
packets.  You can instruct Knockraven to use UDP knocks by specifying
`--protocol udp` (or `-p udp`).  The monitor port is still checked over
TCP because most hidden services (e.g., SSH) are TCP‑based.

```bash
sudo ./knockraven 10.0.0.5 7000 8000 9000 \
    -l 2 -p udp -m 8008 -d 100 -t 500 -c 5
```

This command sends all two‑port UDP sequences (nine combinations) and then
attempts to connect to TCP port `8008` after each sequence.

## Mixed Mode

Mixed mode allows you to test sequences where each knock can be either TCP or
UDP.  For a sequence of length *n* there are `2^n` protocol assignments in
addition to the `port_count^n` port permutations.  This can greatly expand
the search space but is useful for uncovering advanced port‑knocking schemes
that mix protocols.

```bash
sudo ./knockraven 10.0.0.5 1234 5678 \
    -l 2 -p mixed -m 2222 -d 100 -t 500 -c 5
```

In this example Knockraven tests four possible protocol assignments for each
of the two‑port sequences (`tcp/tcp`, `udp/tcp`, `tcp/udp` and `udp/udp`).  If
any combination opens port `2222`, the corresponding port and protocol
combination is displayed, such as `1234/tcp,5678/udp`.

## Performance Considerations

The number of sequences grows exponentially with the length of the knock and
the number of candidate ports.  For example, testing four ports in a
four‑knock sequence yields `4^4 = 256` combinations; five ports in a
five‑knock sequence yields `5^5 = 3125`.  Large combinations may take a
considerable amount of time to complete, even with high concurrency.  Plan
your scans accordingly and focus on the most probable ports first.

Adjust the following parameters to tune performance:

* **Concurrency (`-c`/`--concurrency`)** – The number of sequences tested in
  parallel.  Increase this value to speed up scans, but be mindful of the
  bandwidth and connection limits on your testing environment.
* **Delay (`-d`/`--delay`)** – Time in milliseconds between knocks in a
  sequence.  Too small a delay may cause some daemons to miss knocks; too
  large a delay will slow down the scan.
* **Timeout (`-t`/`--timeout`)** – Maximum time in milliseconds to wait for
  each connection attempt.  Lower values speed up scans but may lead to
  false negatives in high‑latency networks.

## Limitations

* Knockraven currently supports only uniform protocols per scan.  Mixed
  TCP/UDP sequences are not yet implemented.
* Detection relies on the ability to make a TCP connection to the monitor
  port after the knock sequence.  If the target daemon uses a different
  transport (e.g., UDP) to signal success, Knockraven will not detect it.
* Running port‑knocking sequences rapidly and repeatedly may trigger
  intrusion detection systems or cause denial‑of‑service conditions.  Use
  caution and throttle concurrency and delay as appropriate.

## Security Considerations

Port‑knocking is not a replacement for proper authentication.  It provides
an additional barrier by hiding services behind obscure sequences, but
services protected in this way can still be brute forced or sniffed
【688072229527641†L136-L139】.  Moreover, malicious actors have leveraged
port‑knocking for persistence and command and control【804619713130562†L56-L64】.  Always
ensure you have explicit authorization before running Knockraven against any
network and abide by all applicable laws.