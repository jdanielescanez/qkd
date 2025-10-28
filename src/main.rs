use clap::Parser;
use csv::Writer;
use std::collections::HashMap;
use std::process;

use qkd::protocol::{QKDResult, QKD};
use qkd::{build_b92, build_bb84, build_six_state};

/// QKD Simulator CLI
#[derive(Parser, Debug)]
#[command(version, about = "A Quantum Key Distribution simulator developed in Rust", long_about = None)]
struct Args {
    /// Name of protocol to simulate
    #[arg(short, long, required = true, num_args = 1.., value_parser = parse_protocol_tag)]
    protocol: Vec<String>,

    /// Number of qubits to send
    #[arg(short, long, default_values_t = vec![1000], num_args = 1..)]
    number_of_qubits: Vec<usize>,

    /// Rate of intercepted qubits by Eve
    #[arg(short, long, default_values_t = vec![0.0], num_args = 1.., value_parser = parse_rate)]
    interception_rate: Vec<f64>,

    /// Number of repetitions by experiment
    #[arg(short, long, default_value_t = 1)]
    repetitions: usize,

    /// Print results
    #[arg(short, long, default_value_t = false)]
    quiet: bool,

    /// Output CSV file path
    #[arg(short, long)]
    output: Option<String>,
}

/// Struct representing the result of a single QKD experiment.
///
/// # Fields
/// * `id` - Unique identifier for the experiment.
/// * `protocol_tag` - Name of the protocol used.
/// * `n_qubits` - Number of qubits used in the experiment.
/// * `interception_rate` - Probability that Eve intercepted a qubit (0.0 to 1.0).
/// * `result` - The `QKDResult` containing the detailed results of the experiment.
struct ExperimentResult {
    id: String,
    protocol_tag: String,
    n_qubits: usize,
    interception_rate: f64,
    result: QKDResult,
}

/// Executes a series of QKD experiments across different protocols, qubit counts, and interception rates.
///
/// # Arguments
/// * `protocol` - Vector of protocol names to test.
/// * `number_of_qubits` - Vector of qubit counts to use in each experiment.
/// * `interception_rate` - Vector of interception probabilities to test (0.0 to 1.0).
/// * `repetitions` - Number of times to repeat each combination of parameters.
///
/// # Returns
/// A vector of `ExperimentResult` structs, each representing the result of a single experiment.
fn run_experiment(
    protocol: Vec<&QKD>,
    number_of_qubits: Vec<usize>,
    interception_rate: Vec<f64>,
    repetitions: usize,
) -> Vec<ExperimentResult> {
    let protocol_ref = &protocol;
    let number_of_qubits_ref = &number_of_qubits;
    let interception_rate_ref = &interception_rate;
    let repetitions_ref = &repetitions;
    let all_combinations = protocol_ref.iter().flat_map(|protocol_tag| {
        number_of_qubits_ref.iter().flat_map(move |n_qubits| {
            interception_rate_ref
                .iter()
                .flat_map(move |interception_rate| {
                    (0..*repetitions_ref).map(move |repetition| {
                        (protocol_tag, n_qubits, interception_rate, repetition)
                    })
                })
        })
    });

    all_combinations
        .map(|(protocol, &n_qubits, &interception_rate, repetition)| {
            let id = format!(
                "{}_{}_{}-{}",
                protocol.get_name(),
                n_qubits,
                interception_rate,
                repetition
            );

            ExperimentResult {
                id,
                protocol_tag: protocol.get_name(),
                n_qubits,
                interception_rate,
                result: protocol.run(n_qubits, interception_rate),
            }
        })
        .collect()
}

/// Returns a map of available QKD protocol configurations.
///
/// # Returns
/// A `HashMap` where the keys are protocol names and
/// the values are pre-configured `QKD` instances for each protocol.
fn build_all_available_protocols() -> HashMap<String, QKD> {
    HashMap::from([
        ("BB84".to_string(), build_bb84()),
        ("SixState".to_string(), build_six_state()),
        ("B92".to_string(), build_b92()),
    ])
}

fn parse_protocol_tag(s: &str) -> Result<String, String> {
    let allowed_names = build_all_available_protocols();
    if allowed_names.contains_key(s) {
        Ok(s.to_string())
    } else {
        let valid_keys: Vec<_> = allowed_names.keys().collect();
        Err(format!(
            "`{}` is not an allowed protocol. Allowed protocols are: {:?}",
            s, valid_keys
        ))
    }
}

fn parse_rate(s: &str) -> Result<f64, String> {
    if let Ok(rate) = s.parse::<f64>() {
        if 0.0 <= rate && rate <= 1.0 {
            return Ok(rate);
        }
    }
    Err(format!("All rates must be between 0.0 and 1.0"))
}

fn print_aligned_row(columns: &[String]) {
    println!(
        "{:<20} {:<10} {:>15} {:>18} {:>10} {:>20} {:>10} {:>20} {:>10}",
        columns[0],
        columns[1],
        columns[2],
        columns[3],
        columns[4],
        columns[5],
        columns[6],
        columns[7],
        columns[8],
    );
}

fn main() {
    let args = Args::parse();
    let results_header = [
        "id".to_string(),
        "PROTOCOL".to_string(),
        "number_of_qubits".to_string(),
        "interception_rate".to_string(),
        "time_μs".to_string(),
        "is_considered_secure".to_string(),
        "key_length".to_string(),
        "eve_knowledge".to_string(),
        "QBER".to_string(),
    ];

    if !args.quiet {
        print_aligned_row(&results_header);
    } else if args.output.is_none() {
        eprintln!("Error: The `--output` argument is required when `--quiet` is enabled.");
        process::exit(1);
    }

    let mut writer = if let Some(output_path) = &args.output {
        Some(Writer::from_path(output_path).unwrap())
    } else {
        None
    };

    if let Some(w) = &mut writer {
        let _ = w.write_record(&results_header);
    }

    let Args {
        protocol,
        number_of_qubits,
        interception_rate,
        repetitions,
        quiet,
        ..
    } = args;

    let available_protocols = build_all_available_protocols();
    let results = run_experiment(
        protocol
            .iter()
            .map(|tag| &available_protocols[tag])
            .collect(),
        number_of_qubits,
        interception_rate,
        repetitions,
    );

    for ExperimentResult {
        id,
        protocol_tag,
        n_qubits,
        interception_rate,
        result,
    } in results
    {
        let result = [
            id,
            protocol_tag,
            n_qubits.to_string(),
            interception_rate.to_string(),
            result.elapsed_time.to_string(),
            result.is_considered_secure.to_string(),
            result
                .key_length
                .as_ref()
                .map_or("None".to_string(), |v| v.to_string()),
            result.eve_knowledge.to_string(),
            result
                .quantum_bit_error_rate
                .as_ref()
                .map_or("None".to_string(), |v| v.to_string()),
        ];

        if let Some(w) = &mut writer {
            let _ = w.write_record(&result);
        }
        if !quiet {
            print_aligned_row(&result);
        }
    }
}
