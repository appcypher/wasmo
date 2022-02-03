// Copyright 2022 the Gigamono authors. All rights reserved. GPL-3.0 License.
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Compile {},
}

fn main() {
    let _ = Args::parse();
}
