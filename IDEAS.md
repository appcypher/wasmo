### JIT MODE

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

### AOT MODE

- Single-pass compilation only (https://v8.dev/blog/liftoff)
- Compile everything to a single llvm module.
- Compile llvm module to executable native code (as a shared library).
- Write to file.

### AOT EXECUTION MODE

- Set up execution context
    - Memory, Tables
    - Imports
- Load shared library
- Run entry function.
