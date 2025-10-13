use qkd::{run_b92, run_bb84, run_six_state, QResult};

use clap::Parser;
use csv::Writer;
use std::collections::HashMap;
use std::process;

fn get_available_protocols() -> HashMap<String, Box<fn(usize, f64) -> QResult>> {
    HashMap::from([
        (
            "BB84".to_string(),
            Box::new(run_bb84 as fn(usize, f64) -> QResult),
        ),
        (
            "SixState".to_string(),
            Box::new(run_six_state as fn(usize, f64) -> QResult),
        ),
        (
            "B92".to_string(),
            Box::new(run_b92 as fn(usize, f64) -> QResult),
        ),
    ])
}

/// QKD Simulator CLI
#[derive(Parser, Debug)]
#[command(version, about = "A Quantum Key Distribution simulator developed in Rust", long_about = None)]
struct Args {
    /// Name of protocol to simulate
    #[arg(short, long, required = true, num_args = 1.., value_parser = parse_protocol_tag)]
    protocol: Vec<String>,

    /// Number of qubits to send
    #[arg(short, long, default_values_t = vec![1000])]
    number_of_qubits: Vec<usize>,

    /// Rate of intercepted qubits by Eve
    #[arg(short, long, default_values_t = vec![0.0], value_parser = parse_rate)]
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

fn parse_protocol_tag(s: &str) -> Result<String, String> {
    let allowed = get_available_protocols();
    if allowed.contains_key(s) {
        Ok(s.to_string())
    } else {
        let valid_keys: Vec<_> = allowed.keys().collect();
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
        "{:<5} {:<10} {:>15} {:>18} {:>10} {:>20} {:>10} {:>10}",
        columns[0],
        columns[1],
        columns[2],
        columns[3],
        columns[4],
        columns[5],
        columns[6],
        columns[7]
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

    for (protocol_id, protocol_tag) in args.protocol.iter().enumerate() {
        for &n_qubits in &args.number_of_qubits {
            for &interception_rate in &args.interception_rate {
                for id in 0..args.repetitions {
                    let result =
                        get_available_protocols()[protocol_tag](n_qubits, interception_rate);

                    let result_vector = [
                        (id + protocol_id * args.repetitions).to_string(),
                        protocol_tag.to_string(),
                        n_qubits.to_string(),
                        interception_rate.to_string(),
                        result.elapsed_time.as_micros().to_string(),
                        result.is_considered_secure.to_string(),
                        result.key_length.unwrap_or(0).to_string(),
                        result.quantum_bit_error_rate.unwrap_or(-1.0).to_string(),
                    ];

                    if let Some(w) = &mut writer {
                        let _ = w.write_record(&result_vector);
                    }
                    if !args.quiet {
                        print_aligned_row(&result_vector);
                    }
                }
            }
        }
    }
}
