# QKD: Quantum Key Distribution Simulator in Rust

[![Crates.io](https://img.shields.io/crates/v/qkd.svg)](https://crates.io/crates/qkd)
[![Documentation](https://docs.rs/qkd/badge.svg)](https://docs.rs/qkd)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust library and CLI tool for simulating **Quantum Key Distribution (QKD)** protocols, including **BB84**, **Six-State**, and **B92**. This crate provides a flexible and efficient way to simulate quantum key exchange, analyze security metrics, and evaluate the impact of eavesdropping.

---

## Features

- **Multiple QKD Protocols**: Simulate BB84, Six-State, and B92 protocols.
- **Customizable Parameters**: Adjust the number of qubits, interception rate, and repetitions.
- **Security Metrics**: Calculate Quantum Bit Error Rate (QBER), key length, and Eve's knowledge.
- **CLI and Library**: Use as a command-line tool or integrate into your Rust projects.
- **CSV Output**: Export simulation results for further analysis.

---

## Installation

### As a Library

Add `qkd` to your `Cargo.toml`:

```toml
[dependencies]
qkd = "0.0.0"
```

---
## Modules

### `participants`
Defines the `Sender` and `Receiver` structs, which represent Alice and Bob in the QKD protocol. Both use a builder pattern for flexible configuration.

### `protocol`
Contains the core QKD logic, including:
- `QKD`: The main struct to run QKD protocols.
- `QKDResult`: The result of a QKD simulation, including security status, key length, and QBER.
- `PublicDiscussionResult`: The result of the public discussion phase.

### `types`
Defines quantum-related types, such as `Qubit` or `ComplexMatrix`.

### `utils`
Utility functions for quantum operations, such as `suffle_and_split` and basis matrices.

---
## Examples

### Simulate BB84 Protocol
```rust
use qkd::run_bb84;

fn main() {
    let result = run_bb84(1000, 0.1);
    println!("BB84 Result: {:?}", result);
}
```

## Simulate Six-State Protocol

```rust
use qkd::run_six_state;

fn main() {
    let result = run_six_state(1000, 0.1);
    println!("Six-State Result: {:?}", result);
}
```
## Simulate B92 Protocol
```rust
use qkd::run_b92;

fn main() {
    let result = run_b92(1000, 0.1);
    println!("B92 Result: {:?}", result);
}
```

---
## Output

The CLI tool prints results in a formatted table:

```text
id    PROTOCOL   number_of_qubits   interception_rate   time_μs   is_considered_secure   key_length   QBER
0     BB84       1000                0.1                 12345    true                    500          0.05
```

If the `--output` option is provided, results are saved to a CSV file.

---
## License

This project is licensed under the [MIT License](LICENSE).

---
## Contributing

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/jdanielescanez/qkd).
