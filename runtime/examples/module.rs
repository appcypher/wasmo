use std::{env, fs};

use anyhow::Result;
use wasmo_runtime::{Module, Options};

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        return Ok(());
    }

    let module_path = &args[1];
    let wasm_string = wat::parse_str(fs::read_to_string(module_path)?)?;

    Module::new(&wasm_string, Options::default())?;

    Ok(())
}
