use clap::Parser;
use smvx::transpiler::SMVXTranspiler;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Src of smvx files to compile
    #[arg(short, long)]
    src: PathBuf,

    /// Output file path for compiled smv
    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut transpiler = SMVXTranspiler::new();

    let smv_code = transpiler.transpile(cli.src)?;

    if let Some(parent) = cli.output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let mut compiled_file = File::create(&cli.output)?;
    compiled_file.write_all(smv_code.as_bytes())?;

    println!("Successfully transpiled to {}", cli.output.display());
    Ok(())
}
