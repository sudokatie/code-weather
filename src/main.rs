use clap::Parser;
use code_weather::cli::Args;

fn main() {
    let args = Args::parse();
    
    if let Err(e) = code_weather::run(args) {
        eprintln!("error: {}", e);
        std::process::exit(e.exit_code());
    }
}
