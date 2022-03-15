mod test {
    use wasmo_runtime::{Module, Options};

    #[test]
    fn test_parser() {
        env_logger::init();
        let wasm = wat::parse_str(include_str!("../samples/fibonacci.wat")).unwrap();
        let module = Module::new(&wasm, Options::default()).unwrap();
        assert!(true)
    }
}
