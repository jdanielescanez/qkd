use clap::Parser;
use csv::Writer;
use std::collections::HashMap;

use qkd::protocol::{QKDResult, QKD};
use qkd::{build_b92, build_bb84, build_six_state};

// TODO: Separar main y lib
/// QKD Simulator CLI
#[derive(Parser, Debug)]
#[command(
    version,
    about = "A Quantum Key Distribution simulator developed in Rust",
    after_help = "\
EXAMPLES:
    qkd -p BB84,SixState,B92 -s 10,100,1000 -i 0.001,0.01,0.1 -r 5 -n 0,0.001,0.01 -c 0.9999999999,0.999999999 -o output/example.csv",
    long_about = None
)]
struct Args {
    /// List of protocols to simulate (separated by commas)
    #[arg(short, long, required = true, value_parser = parse_protocol_tag, value_delimiter = ',')]
    protocol: Vec<String>,

    /// List of numbers of qubits to send (separated by commas)
    #[arg(short, long, default_values_t = vec![1000], value_delimiter = ',')]
    size: Vec<usize>,

    /// List of rates of intercepted qubits by Eve (separated by commas)
    #[arg(short, long, default_values_t = vec![0.0], value_parser = parse_rate, value_delimiter = ',')]
    interception_rate: Vec<f64>,

    /// Number of repetitions by experiment
    #[arg(short, long, default_value_t = 1)]
    repetitions: usize,

    /// List of probabilities of error in the quantum channel (separated by commas)
    #[arg(short, long, default_values_t = vec![0.0], value_parser = parse_rate, value_delimiter = ',')]
    noise_probability: Vec<f64>,

    /// List of confidence levels to successfully detect eveasdropping (separated by commas)
    #[arg(short, long, default_values_t = vec![1.0 - 10.0_f64.powf(-10.0)], value_parser = parse_rate, value_delimiter = ',')]
    confidence: Vec<f64>,

    /// Output CSV file path
    #[arg(short, long, required = true)]
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
    noise: f64,
    confidence: f64,
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
    protocols: &Vec<&QKD>,
    number_of_qubits: &Vec<usize>,
    interception_rates: &Vec<f64>,
    noise_probabilities: &Vec<f64>,
    confidences: &Vec<f64>,
    repetitions: usize,
) -> Vec<ExperimentResult> {
    let all_combinations = protocols.into_iter().flat_map(|protocol_tag| {
        number_of_qubits.iter().flat_map(move |n_qubits| {
            interception_rates
                .iter()
                .flat_map(move |interception_rate| {
                    noise_probabilities.iter().flat_map(move |noise| {
                        confidences.iter().flat_map(move |confidence| {
                            (0..repetitions).map(move |repetition| {
                                (
                                    protocol_tag,
                                    *n_qubits,
                                    *interception_rate,
                                    *noise,
                                    *confidence,
                                    repetition,
                                )
                            })
                        })
                    })
                })
        })
    });

    all_combinations
        .map(
            |(protocol, number_of_qubits, interception_rate, noise, confidence, repetition)| {
                let id = format!(
                    "{}_{}_{}_{}_{}-{}",
                    protocol.get_name(),
                    number_of_qubits,
                    interception_rate,
                    noise,
                    confidence,
                    repetition
                );

                ExperimentResult {
                    id,
                    protocol_tag: protocol.get_name(),
                    n_qubits: number_of_qubits,
                    interception_rate,
                    noise,
                    confidence,
                    result: protocol.run(number_of_qubits, interception_rate, noise, confidence),
                }
            },
        )
        .collect()
}

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

fn main() {
    let args = Args::parse();

    let mut writer = if let Some(output_path) = &args.output {
        Some(Writer::from_path(output_path).unwrap())
    } else {
        None
    };

    if let Some(w) = &mut writer {
        let header = [
            "id",
            "protocol",
            "number_of_qubits",
            "interception_rate",
            "noise",
            "confidence",
            "time_μs",
            "is_considered_secure",
            "key_length",
            "eve_knowledge",
            "measured_qber",
            "final_key_qber",
        ];

        let _ = w.write_record(&header);
    }

    let Args {
        protocol,
        size,
        interception_rate,
        repetitions,
        noise_probability,
        confidence,
        ..
    } = args;

    let available_protocols = build_all_available_protocols();
    let results = run_experiment(
        &protocol
            .iter()
            .map(|tag| &available_protocols[tag])
            .collect(),
        &size,
        &interception_rate,
        &noise_probability,
        &confidence,
        repetitions,
    );

    for ExperimentResult {
        id,
        protocol_tag,
        n_qubits,
        interception_rate,
        noise,
        confidence,
        result,
    } in results
    {
        let result = [
            id,
            protocol_tag,
            n_qubits.to_string(),
            interception_rate.to_string(),
            noise.to_string(),
            confidence.to_string(),
            result.elapsed_time.to_string(),
            result.is_considered_secure.to_string(),
            result
                .key_length
                .as_ref()
                .map_or("None".to_string(), |v| v.to_string()),
            result.eve_knowledge.to_string(),
            result.measured_qber.to_string(),
            result
                .final_key_qber
                .as_ref()
                .map_or("None".to_string(), |v| v.to_string()),
        ];

        if let Some(w) = &mut writer {
            let _ = w.write_record(&result);
        }
    }
}
