use std::{
    fs,
    io::Write,
    rc::Rc,
    time::{Duration, Instant},
};

use pest::Parser;

// Assuming you have a core module that exposes the VM/Compiler logic
use crate::{
    compiler::{obj::Value, vm::VM, Compiler},
    data::{Data, Response},
    io::InterpretorIO,
    parser::{build_ast_stmt, build_ast_stmts},
    runner::Runner,
    visitor::Visitor,
    AlivescriptParser, Rule,
};

// --- Configuration Constants ---
const WARMUP_RUNS: usize = 5;
const MEASUREMENT_RUNS: usize = 20;
const BENCHMARK_FILE: &str = "bench.as"; // The code we wrote

struct IO {}

impl InterpretorIO for IO {
    fn send(&mut self, data: Data) {
        match data {
            Data::Afficher(s) => println!("{}", s),
            Data::Erreur { texte, ligne } => eprintln!("{}", texte),
            _ => todo!(),
        }
    }
    fn request(&mut self, data: Data) -> Option<Response> {
        match data {
            Data::Afficher(_) => todo!(),
            Data::Erreur { texte, ligne } => todo!(),
            Data::Demander { prompt } => {
                print!("{}", prompt.unwrap_or("Entrez une valeur: ".into()));
                std::io::stdout().flush().unwrap();
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).unwrap();
                Some(Response::Text(line))
            }
            Data::GetFichier(file_path) => {
                let content = std::fs::read_to_string(file_path).ok()?;
                Some(Response::Text(content))
            }
            Data::NotifInfo { msg } => todo!(),
            Data::NotifErr { msg } => todo!(),
        }
    }
}
// --- BENCHMARK DRIVER ---

/// A wrapper function to execute the AliveScript code for one run.
/// This needs to be implemented to call your specific VM logic.
// fn execute_alive_script_a(source_code: &str) {
//     // 1. Compile the source code
//     let compiler = Compiler::new(source_code);
//     let result_stmts = AlivescriptParser::parse(Rule::script, source_code).unwrap();
//     let stmts = build_ast_stmts(result_stmts).unwrap();
//     let closure = compiler.compile(&stmts);
//
//     // 2. Execute the closure
//     let mut vm = VM::new();
//     vm.run(Rc::new(closure)).unwrap();
// }
fn execute_alive_script_a(source_code: &str) {
    // 1. Compile the source code
    let compiler = Compiler::new(source_code);
    let result_stmts = AlivescriptParser::parse(Rule::script, source_code).unwrap();
    let closure = compiler.parse_compile(result_stmts).unwrap();

    // 2. Execute the closure
    let mut vm = VM::new();
    vm.run(Rc::new(closure)).unwrap();
}

fn execute_alive_script_b(source_code: &str) {
    // 1. Compile the source code
    let result_stmts = AlivescriptParser::parse(Rule::script, source_code).unwrap();
    let mut io = IO {};
    let mut visitor = Runner::new(&mut io);
    let stmts = build_ast_stmts(result_stmts).unwrap();
    visitor.visit_body(&stmts);
}

/// Executes the benchmark and returns a vector of measured durations.
fn run_benchmark(source_code: &str, impl_name: &str, func: fn(&str)) -> Vec<Duration> {
    println!("\n--- Benchmarking {} ---", impl_name);

    // 1. WARMUP PHASE (Discarded)
    for i in 0..WARMUP_RUNS {
        func(source_code);
        if i == 0 {
            println!("Warming up ({} runs)...", WARMUP_RUNS);
        }
    }

    // 2. MEASUREMENT PHASE
    let mut times = Vec::with_capacity(MEASUREMENT_RUNS);
    println!("Measuring ({} runs)...", MEASUREMENT_RUNS);

    for _ in 0..MEASUREMENT_RUNS {
        let start = Instant::now();

        // Ensure the result is used to prevent compiler optimization
        let _result = func(source_code);

        let duration = start.elapsed();
        times.push(duration);
    }

    println!("Measurement complete.");
    times
}

// --- ANALYSIS ---

/// Simple manual calculation of average and median.
fn analyze_results(times: &mut [Duration], impl_name: &str) -> Option<Duration> {
    if times.is_empty() {
        println!("{}: No runs recorded.", impl_name);
        return None;
    }

    // Sort for Median calculation
    times.sort();

    // Calculate Total and Average
    let total_time: Duration = times.iter().sum();
    let average = total_time / times.len() as u32;

    // Calculate Median
    let mid = times.len() / 2;
    let median = if times.len() % 2 == 0 {
        // Even number of elements: average the two middle values
        (times[mid - 1] + times[mid]) / 2
    } else {
        // Odd number of elements: the middle value
        times[mid]
    };

    println!("Results for {}", impl_name);
    println!("  -> Total Runs:   {}", times.len());
    println!("  -> Average Time: {:.2?}", average);
    println!("  -> Median Time:  {:.2?}", median);

    Some(median)
}

/// Compares the results of two implementations based on their median times.
fn compare_results(median_a: Duration, median_b: Duration) {
    println!("\n==============================================");
    println!("🚀 Comparative Benchmark Analysis");
    println!("==============================================");

    // Calculate ratio B/A
    let time_a_ns = median_a.as_nanos() as f64;
    let time_b_ns = median_b.as_nanos() as f64;

    if time_a_ns == 0.0 || time_b_ns == 0.0 {
        println!("Cannot perform comparison: One or both median times are zero.");
        return;
    }

    if time_b_ns < time_a_ns {
        // Implementation B is faster (B < A)
        let speedup_ratio = time_a_ns / time_b_ns;
        let percentage_gain = (speedup_ratio - 1.0) * 100.0;

        println!("🟢 IMPL B is FASTER!");
        println!("  -> B is {:.2}x faster than A.", speedup_ratio);
        println!(
            "  -> B represents a {:.1}% performance gain.",
            percentage_gain
        );
    } else {
        // Implementation A is faster or they are roughly equal (B >= A)
        let slowdown_ratio = time_b_ns / time_a_ns;

        if slowdown_ratio > 1.05 {
            // A 5% threshold for "slower"
            let percentage_loss = (slowdown_ratio - 1.0) * 100.0;
            println!("🔴 IMPL B is SLOWER.");
            println!("  -> B is {:.2}x slower than A.", slowdown_ratio);
            println!(
                "  -> B represents a {:.1}% performance loss.",
                percentage_loss
            );
        } else {
            println!("🟡 Performance is roughly EQUAL (within 5% margin).");
        }
    }
    println!("==============================================");
}

pub fn main_benchmark() {
    let source_code = match fs::read_to_string(BENCHMARK_FILE) {
        Ok(code) => code,
        Err(_) => {
            eprintln!(
                "Error: Could not read benchmark file '{}'. Make sure it exists.",
                BENCHMARK_FILE
            );
            return;
        }
    };

    // --- Run and Analyze Impl A ---
    let mut times_a = run_benchmark(
        &source_code,
        "Implementation A (Current VM)",
        execute_alive_script_a,
    );
    let median_a = analyze_results(&mut times_a, "Implementation A").unwrap();

    // --- Run and Analyze Impl B ---
    // NOTE: If Implementation B is a different function/method (e.g., a JIT or a Register VM),
    // you would define a separate execute_alive_script_B function and call it here.
    // Since you only provided one compiler structure, we'll simulate the call.

    // For a simple test, we run the same implementation again to check consistency:
    let mut times_b = run_benchmark(
        &source_code,
        "Implementation B (Hypothetical Optimized VM)",
        execute_alive_script_b,
    );
    let median_b = analyze_results(&mut times_b, "Implementation B").unwrap();

    // --- Final Comparison ---
    // You can compare average times here to summarize the performance gain/loss.
    compare_results(median_a, median_b);
}
