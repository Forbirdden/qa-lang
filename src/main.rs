mod config;
mod preprocessor;
mod compiler;

use std::fs;
use std::process::Command;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use clap::Parser;
use colored::*;

fn main() {
    let cfg = config::Config::parse();
    let mut loaded = HashSet::new();

    println!("{}", "⚓ Loading...".cyan());
    let source = preprocessor::load_source(&cfg.input, &mut loaded);
    let final_src = compiler::compile(source);

    if cfg.verbose {
        println!("{}\n{}", "--- GENERATED C CODE ---".yellow(), final_src);
    }

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let temp_c = format!("temp_{}.c", nanos);
    fs::write(&temp_c, &final_src).expect("Failed to write temp C file");

    let output = cfg.final_output();
    println!("{}", format!("🚀 Compiling to {}...", output).green());

    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let status = Command::new(cc)
        .args([&temp_c, "-o", &output, "-Os", "-s"])
        .status()
        .expect("Failed to run C compiler");

    if !cfg.keep_temp { let _ = fs::remove_file(&temp_c); }

    if status.success() {
        println!("{}", "✅ Build successful!".bold().green());
        if cfg.run || cfg.test {
            let run_path = if output.contains('/') || output.contains('\\') { output.clone() } else { format!("./{}", output) };
            let _ = Command::new(run_path).status();
            if cfg.test { let _ = fs::remove_file(&output); }
        }
    } else {
        eprintln!("{}", "❌ Compilation failed.".red());
    }
}
