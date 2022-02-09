### LAZY MODE

- Single-pass compilation only (https://v8.dev/blog/liftoff)
- Set up execution context
    - Memory, Tables
    - Imports
- Parse wasm blob
    - Get metadata from an initial linear pass
        - Give every function a canonical address
    - Break function codes up so they are locatable with canonical addressing
    - Compile entry function
        - Calls to other functions are trampolined
- Run entry function.
- When an unresolved function is called (taking cues from GOT-PLT implementation)
    - Jumps PLT which jumps using address to Compiler.
    - Compile code.
    - Rewrite GOT address to function code.
    - Run function code.
- Thankfully ORC allows us to do some of these. Here are other ideas I have
    - Using metadata gotten from first linear pass.
    - Speculatively compile next function call (if not already compiled) on as separate core.

### EAGER MODE

- Single-pass compilation only (https://v8.dev/blog/liftoff)
- Compile everything to a single llvm module.
- Serialize wasm module.
- Write to file.


### PROPOSED API

```rs
let imports = Imports::default(/* memories, tables, globals, functions */)?;
let module = Module::new(&bytes, CompileMode::Eager)?; // Compiles with unresolved symbols. Creates trampolines.
let instance = Instance::new(&module, &imports)?; // Links memory pieces. Makes imported functions where accessible.
```

```rs
module.dump(); // Dumps serialized module to Vec<u8> or &[u8].
```
