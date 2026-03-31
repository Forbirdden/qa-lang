use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "A Qa lang compiler", long_about = None)]
pub struct Config {
    /// Qa source file
    pub input: String,

    /// Compiled binary name
    pub output: Option<String>,

    /// Print transpiled C code
    #[arg(short, long)]
    pub verbose: bool,

    /// Keep transpiled C code
    #[arg(short, long)]
    pub keep_temp: bool,

    /// Auto-run after binary compiled
    #[arg(short, long)]
    pub run: bool,

    /// Auto-run after binary compiled and remove binary after it's finished
    #[arg(short, long)]
    pub test: bool,
}

impl Config {
    pub fn final_output(&self) -> String {
        self.output.clone().unwrap_or_else(|| {
            if cfg!(windows) {
                self.input.replace(".qa", ".exe")
            } else {
                self.input.replace(".qa", "")
            }
        })
    }
}
