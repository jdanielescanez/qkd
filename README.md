# QKD: Quantum Key Distribution Library in Rust

[![Crates.io](https://img.shields.io/crates/v/qkd.svg)](https://crates.io/crates/qkd)
[![Documentation](https://docs.rs/qkd/badge.svg)](https://docs.rs/qkd)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

<div align="center">
  <a href="https://github.com/jdanielescanez/qkd">
    <img src="https://github.com/jdanielescanez/qkd/blob/main/assets/logo.png?raw=true" alt="qkd Logo" width="150">
  </a>

  <h3 align="center">qkd</h3>

  <p align="center">
    <a href="https://docs.rs/qkd/latest/qkd/"><strong>Explore the docs »</strong></a>
  </p>
</div>

A Rust library for simulating **Quantum Key Distribution (QKD)** protocols, including **BB84**, **Six-State**, **B92**, the entanglement-based **BBM92** and **E91** (secured by a Bell/CHSH inequality test rather than an error-rate threshold), and **your own protocols**. This crate provides a flexible and efficient way to simulate quantum key exchange, analyze security metrics, and evaluate the impact of eavesdropping.

---

## Features

- **Multiple QKD Protocols**: Simulate BB84, Six-State, B92, BBM92 and E91 (both entanglement-based), and your own protocols.
- **Entanglement support**: A joint multi-qubit `QuantumState` representation backs every qubit, so entanglement-based protocols are first-class, not bolted on.
- **Pluggable security proofs**: The security check itself is injectable, not just the measurement bases — E91's CHSH test lives side by side with the QBER threshold used by every other protocol.
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
Defines the `Sender` and `Receiver` structs, representing Alice, Bob, and Eve in the QKD protocol. Uses a builder pattern for flexible configuration of quantum bases and participant behaviors. `Sender::prepare` obtains the qubit that continues towards Bob along with Alice's classical value and the basis she used — whether by preparing it directly (BB84-style) or by measuring her own half of an entangled pair (BBM92-style); see [Build your own protocols](#build-your-own-protocols).

### `protocol`
Implements the core Quantum Key Distribution protocols. Contains the main `QKD` struct, protocol execution logic, and result types including `QKDResult` and `PublicDiscussionResult`. `QKD::check_security` is injectable just like `public_basis_discussion`: the default is a QBER-vs-`noise` threshold, but a protocol whose security proof isn't based on an error rate — like E91's CHSH test — can supply its own instead; see [Build your own protocols](#build-your-own-protocols).

### `constants`
Contains fundamental quantum constant matrices (`I`, `H`, `X`, `H_Y`) and `rotation(theta)`, a parameterized measurement-basis change used by E91's arbitrary-angle bases.

### `rng`
Provides the global seedable random number generator used throughout the simulations. Call `set_global_seed` for reproducible results.

### Entanglement internals
Every qubit is backed by a joint `QuantumState` (a vector of `2^n` complex amplitudes for `n` qubits), with `Qubit` acting as a lightweight handle into it rather than owning any amplitude itself — this is what allows a `Qubit` to be part of an entangled, multi-qubit state. This machinery is internal (used by `build_bbm92`); it is not yet part of the public API for building your own entangled protocols.

---
## Example

### Execution

```rust
use qkd::{build_bb84, build_six_state, build_b92, build_bbm92, build_e91};

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

    // BBM92: the entanglement-based counterpart of BB84. A Bell pair is created each
    // round instead of Alice preparing and sending a qubit directly, but it behaves
    // identically from the outside (same bases, same statistics, same metrics).
    let result = build_bbm92().run(NUMBER_OF_QUBITS, INTERCEPTION_RATE, NOISE, CONFIDENCE);
    println!("BBM92 Result: {:?}", result);

    // E91: secured by violating a Bell (CHSH) inequality instead of an error-rate
    // threshold. `measured_qber` holds the CHSH parameter |S| for this protocol, not
    // a bit error rate — see "Understanding E91" below. A larger qubit count helps
    // here, since each of the 4 CHSH basis-pair combinations only occurs on ~1/9 of
    // the rounds.
    let result = build_e91().run(100_000, INTERCEPTION_RATE, NOISE, CONFIDENCE);
    println!("E91 Result: {:?}", result);
}
```

These structs contain the results of a QKD protocol execution, including execution time, security status, final key length, quantum bit error rate (QBER) for both the final key and public values, and the estimated fraction of the key known by an eavesdropper (Eve).

### Understanding `noise` and `confidence`

`noise` is not just a lower bound on the tolerated error rate: it is the *exact* per-bit error
probability that the security check assumes to be true in the absence of eavesdropping (the
null hypothesis of a one-sided statistical test). The tolerance around it scales with
`sqrt(noise * (1 - noise))`, and `confidence` controls how many standard deviations of that
tolerance are allowed.

**When `noise = 0.0` (as in the example above), that tolerance is exactly `0.0`, so `confidence`
has no effect at all**: the check degenerates into requiring the publicly disclosed values to
match *exactly*. This is intentional — a channel declared to have zero noise should never
produce an error — but it also means a single, one-off disturbance (from Eve or otherwise) is
enough to make `is_considered_secure` `false`. If you want `confidence` to provide real slack,
set `noise` to a small positive value that reflects your channel's expected imperfection
(e.g. `0.01`) instead of `0.0`.

### Understanding `eve_knowledge`

`eve_knowledge` is a **lower bound**, not an exact measurement, on the fraction of the final
key Eve knows. A final-key bit is only credited to Eve when: she intercepted that round, *and*
Alice's and Bob's values for it agree (i.e. it did not contribute to `final_key_qber`), *and*
her measured value matches theirs. Bits where Alice and Bob disagree are excluded from the
numerator (whether the disagreement came from noise or from Eve's own interference is not
distinguished), even though Eve may have measured those bits correctly too — so
`eve_knowledge` can understate Eve's true knowledge whenever `final_key_qber > 0.0`.

`eve_knowledge` is also only meaningful when `is_considered_secure` is `true`: if the protocol
is aborted, it is left at its default `0.0` instead of becoming unavailable like `key_length`
and `final_key_qber` do — so a `0.0` value does not by itself mean "Eve learned nothing", it
may just mean the run was aborted before this metric was computed. Always check
`is_considered_secure` before interpreting `eve_knowledge`.

### Understanding E91

E91 (Ekert 1991) is secured differently from every other protocol here: instead of checking
that the publicly disclosed error rate stays below a threshold, it checks that a **Bell/CHSH
inequality** is violated — a property no classical (local hidden-variable) process can achieve,
but quantum entanglement can.

Each round, Alice and Bob independently pick one of 3 measurement angles. Two angle
combinations line up exactly (giving perfectly correlated outcomes, used for the key); four
more combinations are combined into the CHSH parameter
`S = E(a0,b1) - E(a0,b2) + E(a1,b1) + E(a1,b2)`, where each `E(ai,bj)` is the measured
correlation for that angle pair. Classically, `|S| ≤ 2`; quantum mechanics allows up to
`2 * sqrt(2) ≈ 2.828` (the Tsirelson bound) for the entangled pairs this crate produces.
`is_considered_secure` checks whether the *lower* confidence bound of `|S|` still exceeds `2.0`.

Because `QKDResult` is shared across every protocol, `measured_qber` holds `|S|` for E91, not a
bit error rate — a value around `2.0` to `2.828` is expected without eavesdropping, and a value
dropping towards (or below) `2.0` indicates interference. Each of the 4 CHSH angle pairs only
occurs on about 1/9 of the rounds, so E91 needs a larger `number_of_qubits` than the other
protocols for a statistically stable `|S|` estimate (tens of thousands, not hundreds).

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
    let prepare_b92 = Box::new(|posible_basis: &Vec<ComplexMatrix>| {
        let qubit = Qubit::create_basis_state(false);
        let (basis_id, matrix) = rand_choose(posible_basis.iter().enumerate().collect());
        qubit.apply_local_gate(matrix);
        (qubit, false, basis_id)
    });

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

`Sender::prepare` always returns the qubit that continues towards Bob, Alice's classical
value, and the basis index she used — fusing what would otherwise be two separate steps
("prepare" and "choose a basis") into one. This is what lets entanglement-based protocols
share the exact same execution pipeline: `build_bbm92`'s `prepare` creates a Bell pair,
applies a random basis to *Alice's own* half and measures it, then returns *Bob's* half —
already left in exactly the right state by entanglement — along with Alice's value and basis:

```rust
pub fn build_bbm92() -> QKD {
    let alice = Sender::builder()
        .posible_basis(vec![I, H])
        .prepare(Box::new(prepare_bbm92))
        .build();
    let bob = Receiver::builder().posible_basis(vec![I, H]).build();

    QKD::builder().name("BBM92".to_string()).alice(alice).bob(bob).build()
}

fn prepare_bbm92(posible_basis: &Vec<ComplexMatrix>) -> (Qubit, bool, usize) {
    let (alice_qubit, bob_qubit) = Qubit::create_entangled_state();
    let (basis_id, matrix) = rand_choose(posible_basis.iter().enumerate().collect());
    alice_qubit.apply_local_gate(matrix);
    let alice_value = alice_qubit.measure();
    (bob_qubit, alice_value, basis_id)
}
```

Note `Qubit::create_entangled_state` is currently internal to the crate (used to build
`build_bbm92`/`build_e91`), not part of the public API — external code can build custom
"prepare and send" protocols like B92 above, but not yet custom entanglement-based ones.

For a protocol whose security proof isn't a QBER threshold — like E91's — `QKD::check_security`
is injectable the same way `public_basis_discussion` is. It receives the full
`PublicDiscussionResult` (not just the publicly disclosed values), because a check like CHSH
needs to know which basis pair produced each round, not just a flat list of disclosed bits:

```rust
QKD::builder()
    .name("E91".to_string())
    .alice(alice)
    .bob(bob)
    .eve(eve)
    .public_basis_discussion(Box::new(public_basis_discussion_e91))
    .check_security(Box::new(check_security_e91))
    .build()
```


---
## License

This project is licensed under the [MIT License](LICENSE).

---
## Contributing

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/jdanielescanez/qkd).
