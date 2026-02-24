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
    /// Classify a PDF lab report
    Classify {
        /// Path to the PDF file
        pdf_file: PathBuf,

        /// Custom JSON rule file(s)
        #[arg(short, long = "rules", value_name = "FILE")]
        rules: Vec<PathBuf>,

        /// Predefined ruleset(s): nv, fa, ifa (default: nv if no --rules given)
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
        Commands::Classify {
            pdf_file,
            rules,
            preset,
            output,
            show_all,
            verbose,
        } => commands::classify::run(pdf_file, rules, preset, &output, show_all, verbose),
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
