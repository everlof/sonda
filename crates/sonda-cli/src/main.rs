mod commands;
mod output;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "sonda",
    version,
    about = "Waste classification tool for contaminated soil and asphalt"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a lab report (PDF or Sweco XLSX) into structured data (without classifying)
    Parse {
        /// Path to PDF or Sweco XLSX file
        input_file: PathBuf,

        /// Output format: table (default) or json
        #[arg(short, long, default_value = "table")]
        output: String,

        /// Write parsed output to a JSON file
        #[arg(short = 'O', long = "out", value_name = "FILE")]
        out: Option<PathBuf>,
    },
    /// Classify a lab report (PDF or pre-parsed JSON)
    Classify {
        /// Path to PDF or pre-parsed JSON file
        input_file: PathBuf,

        /// Custom JSON rule file(s)
        #[arg(short, long = "rules", value_name = "FILE")]
        rules: Vec<PathBuf>,

        /// Predefined ruleset(s): nv, asfalt, fa (default: all presets if no --rules/--preset given)
        #[arg(short, long = "preset", value_name = "NAME")]
        preset: Vec<String>,

        /// Output format: table (default) or json
        #[arg(short, long, default_value = "table")]
        output: String,

        /// Show all substances, not just exceedances
        #[arg(long)]
        show_all: bool,

        /// Show detailed per-substance reasoning
        #[arg(long)]
        verbose: bool,
    },
    /// Manage and inspect rulesets
    Rules {
        #[command(subcommand)]
        action: RulesAction,
    },
}

#[derive(Subcommand)]
enum RulesAction {
    /// List predefined rulesets
    List,
    /// Explain a ruleset in plain language
    Explain {
        /// Preset name (e.g., "nv")
        preset: String,
    },
    /// Print the JSON schema with field descriptions and example
    Schema,
    /// Validate a custom rule file
    Validate {
        /// Path to JSON rule file
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Parse {
            input_file,
            output,
            out,
        } => commands::parse::run(input_file, &output, out),
        Commands::Classify {
            input_file,
            rules,
            preset,
            output,
            show_all,
            verbose,
        } => commands::classify::run(input_file, rules, preset, &output, show_all, verbose),
        Commands::Rules { action } => match action {
            RulesAction::List => commands::rules::list(),
            RulesAction::Explain { preset } => commands::rules::explain(&preset),
            RulesAction::Schema => commands::rules::schema(),
            RulesAction::Validate { file } => commands::rules::validate(&file),
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
