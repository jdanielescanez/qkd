# QKD: Quantum Key Distribution Library in Rust

[![Crates.io](https://img.shields.io/crates/v/qkd.svg)](https://crates.io/crates/qkd)
[![Documentation](https://docs.rs/qkd/badge.svg)](https://docs.rs/qkd)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust library for simulating **Quantum Key Distribution (QKD)** protocols, including **BB84**, **Six-State**, **B92** and **your own protocols**. This crate provides a flexible and efficient way to simulate quantum key exchange, analyze security metrics, and evaluate the impact of eavesdropping.

---

## Features

- **Multiple QKD Protocols**: Simulate BB84, Six-State, B92 and your own protocols.
- **Customizable Parameters**: Adjust the number of qubits, interception rate, noise and confidence.
- **Security Metrics**: Calculate Quantum Bit Error Rate (QBER), key length, and Eve's knowledge.
- **Library**: Integrate into your Rust projects.

---

## Installation

```bash
cargo add qkd
```

---
## Modules

### `participants`
Defines the `Sender` and `Receiver` structs, representing Alice, Bob, and Eve in the QKD protocol. Uses a builder pattern for flexible configuration of quantum bases and participant behaviors.

### `protocol`
Implements the core Quantum Key Distribution protocols. Contains the main `QKD` struct, protocol execution logic, and result types including `QKDResult` and `PublicDiscussionResult`.

### `constants`
Contains fundamental quantum constant matrices. Provides predefined quantum gates (`I`, `H`, `X`, `H_Y`) used in QKD protocols for state transformations and measurements.

### `helpers`
Provides utility functions and common quantum operations. Contains mathematical utilities and helper functions like `shuffle_and_split` for protocol execution and data processing.

---
## Example

### Execution

```rust
use qkd::{build_bb84, build_six_state, build_b92};

const NUMBER_OF_QUBITS: usize = 1000;
const INTERCEPTION_RATE: f64 = 0.01;
const NOISE: f64 = 0.0;
const CONFIDENCE: f64 = 0.9999999999;

fn main() {
    let result = build_bb84().run(NUMBER_OF_QUBITS, INTERCEPTION_RATE, NOISE, CONFIDENCE);
    println!("BB84 Result: {:?}", result);

    let result = build_six_state().run(NUMBER_OF_QUBITS, INTERCEPTION_RATE, NOISE, CONFIDENCE);
    println!("Six-State Result: {:?}", result);

    let result = build_b92().run(NUMBER_OF_QUBITS, INTERCEPTION_RATE, NOISE, CONFIDENCE);
    println!("B92 Result: {:?}", result);
}
```

These structs contain the results of a QKD protocol execution, including execution time, security status, final key length, quantum bit error rate (QBER) for both the final key and public values, and the estimated fraction of the key known by an eavesdropper (Eve).

### Build your own protocols

Observe how simple it is to define your own protocol using this example of the Six-State protocol.

```rust
pub fn build_six_state() -> QKD {
    let alice = Sender::builder().posible_basis(vec![I, H, H_Y]).build();
    let bob = Receiver::builder()
        .posible_basis(vec![I, H, H_Y.invert().unwrap()])
        .build();
    let eve = Receiver::builder()
        .posible_basis(vec![I, H, H_Y.invert().unwrap()])
        .build();

    QKD::builder()
        .name("SixState".to_string())
        .alice(alice)
        .bob(bob)
        .eve(eve)
        .build()
}
```

It is also possible to define your own implementation of the public discussion phase of the rules, as in this other example from protocol B92:

```rust
fn public_basis_discussion_b92(results: &Vec<QExecutionResult>) -> PublicDiscussionResult {
    let mut results = results.clone();
    let bob_values: Vec<bool> = results.iter().map(|x| x.bob_value).collect();

    let conclusive_indexes = bob_values
        .iter()
        .enumerate()
        .filter_map(|(i, &value)| if value { Some(i) } else { None })
        .collect::<Vec<usize>>();

    results.iter_mut().enumerate().for_each(|(i, result)| {
        if conclusive_indexes.contains(&i) {
            result.bob_value = (1 - result.bob_basis) == 1;
        }
        result.alice_value = result.alice_basis == 1;
    });

    let (indexes_to_check, indexes_to_key) = shuffle_and_split(conclusive_indexes);
    let (alice_public_values, bob_public_values) = indexes_to_check
        .iter()
        .map(|&i| (results[i].alice_value, results[i].bob_value))
        .unzip();

    PublicDiscussionResult {
        alice_public_values,
        bob_public_values,
        indexes_to_key,
        results,
    }
}

pub fn build_b92() -> QKD {
    let prepare_b92 = Box::new(|| (Qubit::new(), false));

    let alice = Sender::builder()
        .posible_basis(vec![I, H])
        .prepare(prepare_b92)
        .build();
    let bob = Receiver::builder().posible_basis(vec![I, H]).build();

    QKD::builder()
        .name("B92".to_string())
        .alice(alice)
        .bob(bob)
        .public_basis_discussion(Box::new(public_basis_discussion_b92))
        .build()
}
```


---
## License

This project is licensed under the [MIT License](LICENSE).

---
## Contributing

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/jdanielescanez/qkd).
