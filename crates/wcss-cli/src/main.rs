mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wcss", version, about = "WCSS - Web Compiler Style Sheets CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile WCSS files to CSS
    Build {
        /// Input file or glob pattern
        #[arg(default_value = "**/*.wcss")]
        input: String,

        /// Output directory or file
        #[arg(short, long)]
        output: Option<String>,

        /// Minify CSS output
        #[arg(short, long)]
        minify: bool,

        /// Generate source maps (inline or external)
        #[arg(short, long, value_name = "TYPE")]
        source_maps: Option<String>,

        /// Enable Typed OM runtime
        #[arg(long)]
        typed_om: bool,

        /// Enable tree shaking
        #[arg(long)]
        tree_shaking: bool,
    },

    /// Watch WCSS files and recompile on changes
    Watch {
        /// Input file or glob pattern
        #[arg(default_value = "**/*.wcss")]
        input: String,

        /// Output directory or file
        #[arg(short, long)]
        output: Option<String>,

        /// Minify CSS output
        #[arg(short, long)]
        minify: bool,

        /// Generate source maps (inline or external)
        #[arg(short, long, value_name = "TYPE")]
        source_maps: Option<String>,

        /// Enable Typed OM runtime
        #[arg(long)]
        typed_om: bool,

        /// Enable tree shaking
        #[arg(long)]
        tree_shaking: bool,
    },

    /// Format WCSS files
    Format {
        /// Input file or glob pattern
        #[arg(default_value = "**/*.wcss")]
        input: String,

        /// Write formatted output back to file
        #[arg(short, long)]
        write: bool,
    },

    /// Compile W3C Design Tokens to platform-specific code
    Tokens {
        /// Input W3C Design Tokens JSON file
        input: String,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: String,

        /// Target platform (css, ios, android, android-kotlin, flutter, typescript, docs)
        #[arg(short, long, default_value = "css")]
        platform: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build {
            input,
            output,
            minify,
            source_maps,
            typed_om,
            tree_shaking,
        } => commands::build::run(&input, output.as_deref(), minify, source_maps.as_deref(), typed_om, tree_shaking),

        Commands::Watch {
            input,
            output,
            minify,
            source_maps,
            typed_om,
            tree_shaking,
        } => commands::watch::run(&input, output.as_deref(), minify, source_maps.as_deref(), typed_om, tree_shaking),

        Commands::Format { input, write } => commands::format::run(&input, write),

        Commands::Tokens {
            input,
            output,
            platform,
        } => commands::tokens::run(&input, &output, &platform),
    };

    if let Err(e) = result {
        eprintln!("{}: {}", colored::Colorize::red("error"), e);
        std::process::exit(1);
    }
}
