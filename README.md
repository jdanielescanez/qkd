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

This structs contain the results of a QKD protocol run, including execution time, security status, final key length, quantum bit error rate (QBER), and Eve's estimated knowledge of the key.


### As a binary

Execute the simulator using the following command:

```
qkd --protocol <protocol> [OPTIONS]
```

#### Options

| Option                     | Description                                                                                     | Default Value |
|----------------------------|-------------------------------------------------------------------------------------------------|---------------|
| `--protocol`, `-p`         | QKD protocol to simulate (`BB84`, `SixState`, `B92`) [required]                                | -              |
| `--number-of-qubits`, `-n` | Number of qubits to send in the simulation.                                                    | `1000`        |
| `--interception-rate`, `-i`| Interception rate of qubits by Eve (value between `0.0` and `1.0`).                           | `0.0`         |
| `--repetitions`, `-r`      | Number of repetitions of the experiment.                                                       | `1`           |
| `--quiet`, `-q`             | Suppress console output.                                                                        | `false`       |
| `--output`, `-o`           | Path to the CSV file where results will be saved (required if `--quiet` is enabled).            | None          |
| `--help`, `-h` | Print help |
| `--version`, `-V` | Print version |

#### Examples

Run the BB84 protocol with default parameters:
```
qkd --protocol BB84
```

The terminal will display the following result:

```
id    PROTOCOL   number_of_qubits  interception_rate    time_μs is_considered_secure key_length       QBER
0     BB84                  1000                  0       1709                 true        250          0
```

---

Run the B92 protocol with 2000 qubits, an interception rate of 5%, and 3 repetitions:
```
qkd --protocol B92 --number-of-qubits 2000 --interception-rate 0.05 --repetitions 3 --quiet --output output/example.csv
```

The terminal will not display any results, but it will have generated the following file in the [specified path](./output/example.csv):

```
id,PROTOCOL,number_of_qubits,interception_rate,time_μs,is_considered_secure,key_length,QBER
0,B92,2000,0.05,6836,false,0,-1
1,B92,2000,0.05,8979,false,0,-1
2,B92,2000,0.05,11684,false,0,-1
```
---

Run multiple QKD protocols (BB84, SixState, and B92) with different parameters in a single execution:
```
cargo run -- -p BB84 SixState B92 -n 100 1000 -i 0.001 0.01 -q -o output/complete_example.csv
```

The terminal will not display any results, but it will have generated the following file in the [specified path](./output/complete_example.csv):

```
id,PROTOCOL,number_of_qubits,interception_rate,time_μs,is_considered_secure,key_length,QBER
0,BB84,100,0.001,245,true,24,0
1,BB84,100,0.01,128,false,0,-1
2,BB84,1000,0.001,4412,true,242,0
3,BB84,1000,0.01,1383,true,248,0.004032258064516129
4,SixState,100,0.001,135,true,17,0
5,SixState,100,0.01,128,true,12,0
6,SixState,1000,0.001,1577,true,151,0
7,SixState,1000,0.01,1292,true,167,0.011976047904191617
8,B92,100,0.001,127,true,17,0
9,B92,100,0.01,114,true,11,0
10,B92,1000,0.001,2595,true,121,0
11,B92,1000,0.01,1961,true,125,0
```

---
## License

This project is licensed under the [MIT License](LICENSE).

---
## Contributing

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/jdanielescanez/qkd).
