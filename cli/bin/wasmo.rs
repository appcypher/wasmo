use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run {},
}

fn main() {
    let _ = Args::parse();
    println!("wasmo: The Wasmo CLI");
}
