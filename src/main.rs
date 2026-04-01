mod config;
mod preprocessor;
mod compiler;

use std::fs;
use std::process::{Command, ExitStatus};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::{Path, PathBuf};
use clap::Parser;
use colored::*;
use include_dir::{include_dir, Dir};

static ZIG_BUNDLE: Dir = include_dir!("$ZIG_BUNDLE_PATH");

fn main() {
    let cfg = config::Config::parse();
    let mut loaded = HashSet::new();

    if cfg.verbose { println!("{}", "⚓ Loading...".cyan()); }
    
    let source = preprocessor::load_source(&cfg.input, &mut loaded);
    let final_src = compiler::compile(source);

    if cfg.verbose {
        println!("{}\n{}", "--- GENERATED C CODE ---".yellow(), final_src);
    }

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let temp_c = format!("temp_{}.c", nanos);
    fs::write(&temp_c, &final_src).expect("Failed to write temp C file");

    let output = cfg.final_output();
    
    let status = try_compile(&temp_c, &output, cfg.verbose);

    if !cfg.keep_temp { let _ = fs::remove_file(&temp_c); }

    match status {
        Some(s) if s.success() => {
            if !cfg.test { println!("{}", "✅ Build successful!".bold().green()); }
            if cfg.run || cfg.test {
                let run_path = if output.contains('/') || output.contains('\\') { 
                    output.clone() 
                } else { 
                    format!("./{}", output) 
                };
                let _ = Command::new(run_path).status();
                if cfg.test { let _ = fs::remove_file(&output); }
            }
        }
        Some(s) => {
            eprintln!("{}", format!("❌ Compilation failed with exit status: {}.", s).red());
            std::process::exit(s.code().unwrap_or(1));
        }
        None => {
            eprintln!("{}", "❌ Error: C compiler was NOT FOUND.".red());
            std::process::exit(1);
        }
    }
}

fn try_compile(input: &str, output: &str, verbose: bool) -> Option<ExitStatus> {
    let qack_dir = dirs::home_dir()?.join(".qack").join("bin");
    
    let mut zig_exe = find_zig_binary(&qack_dir);

    if zig_exe.is_none() {
        if verbose { println!("{}", "🚚 First run detected, build will be longer...".yellow()); }
        fs::create_dir_all(&qack_dir).ok()?;
        ZIG_BUNDLE.extract(&qack_dir).ok()?;
        zig_exe = find_zig_binary(&qack_dir);
    }

    if let Some(exe) = zig_exe {
        if verbose { println!("{}", format!("📦 Using embedded C compiler: {}", exe.display()).blue()); }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&exe, fs::Permissions::from_mode(0o755));
        }

        let mut cmd = Command::new(&exe);
        
        let qack_root = qack_dir.parent().unwrap();
        let global_cache = qack_root.join("cache");
        let local_cache = qack_root.join("local_cache");
        fs::create_dir_all(&global_cache).ok();
        fs::create_dir_all(&local_cache).ok();

        cmd.args(["cc", input, "-o", output, "-Os", "-s"]);
        
        cmd.env("ZIG_GLOBAL_CACHE_DIR", &global_cache);
        cmd.env("ZIG_LOCAL_CACHE_DIR", &local_cache);
        
        let lib_path = exe.parent().unwrap().join("lib");
        cmd.env("ZIG_LIB_DIR", &lib_path);

        if verbose { cmd.arg("-v"); }

        return cmd.status().ok();
    }

    let cc_env = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    if verbose { println!("{}", format!("🔍 Using system compiler: {}", cc_env).blue()); }

    let mut cmd = Command::new(cc_env);
    cmd.args([input, "-o", output, "-Os", "-s"]);
    if verbose { cmd.arg("-v"); }
    cmd.status().ok()
}

fn find_zig_binary(path: &Path) -> Option<PathBuf> {
    if !path.exists() { return None; }
    for entry in fs::read_dir(path).ok()? {
        let entry = entry.ok()?;
        let p = entry.path();
        if p.is_dir() {
            if let Some(found) = find_zig_binary(&p) { return Some(found); }
        } else {
            let name = p.file_name()?.to_string_lossy();
            if name == "zig" || name == "zig.exe" { return Some(p); }
        }
    }
    None
}
