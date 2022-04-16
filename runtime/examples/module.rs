use wasmo_runtime::{Module, Options};

fn main() {
    env_logger::init();
    let wasm = wat::parse_str(include_str!("../../tests/samples/experiment.wat")).unwrap();
    Module::new(&wasm, Options::default()).unwrap();
}
