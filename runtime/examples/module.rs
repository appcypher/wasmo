use std::{env, fs};

use wasmo_runtime::{Module, Options};

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        return;
    }

    let module_path = &args[1];
    let wasm_string = wat::parse_str(fs::read_to_string(module_path).unwrap()).unwrap();

    Module::new(&wasm_string, Options::default()).unwrap();
}
