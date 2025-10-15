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

### As a library

```bash
cargo add qkd
```

### As a binary

```bash
cargo install qkd
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
Utility functions for quantum operations, such as `shuffle_and_split` and basis matrices.

---
## Example

### As a library
```rust
use qkd::{run_bb84, run_six_state, run_b92};

fn main() {
    let result = run_bb84(1000, 0.01);
    println!("BB84 Result: {:?}", result);

    let result = run_six_state(1000, 0.01);
    println!("Six-State Result: {:?}", result);
    
    let result = run_b92(1000, 0.01);
    println!("B92 Result: {:?}", result);
}
```


### As a binary

Execute the simulator using the following command:

```
qkd --protocol <protocol> [OPTIONS]
```

#### Options

| Option                     | Description                                                                                     | Default Value |
|----------------------------|-------------------------------------------------------------------------------------------------|---------------|
| `--protocol`, `-p`         | QKD protocol to simulate (`BB84`, `SixState`, `B92`).                                          | Required      |
| `--number-of-qubits`, `-n` | Number of qubits to send in the simulation.                                                    | `1000`        |
| `--interception-rate`, `-i`| Interception rate of qubits by Eve (value between `0.0` and `1.0`).                           | `0.0`         |
| `--repetitions`, `-r`      | Number of repetitions of the experiment.                                                       | `1`           |
| `--quiet`, `-q`             | Suppress console output.                                                                        | `false`       |
| `--output`, `-o`           | Path to the CSV file where results will be saved (required if `--quiet` is enabled).            | None          |


#### Examples

Run the BB84 protocol with default parameters:
```
qkd --protocol BB84
```

Run the B92 protocol with 2000 qubits, an interception rate of 5%, and 3 repetitions:
```
qkd --protocol B92 --number-of-qubits 2000 --interception-rate 0.05 --repetitions 3
```

---
## License

This project is licensed under the [MIT License](LICENSE).

---
## Contributing

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/jdanielescanez/qkd).
