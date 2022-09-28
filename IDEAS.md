### GOALS

In this order.

1. Simple implementation.
2. Single-pass compilation.
3. Serializable.
4. Progressive optimisation.

##

### MODES

Wasmo supports two modes of compilation:

1. Highly-Optimised Mode
2. Lift-Off Mode

The highly-optimised mode compiles the wasm binary once into executable code with important optimisations enabled. This is useful when the module is going to be cached for subsequent reuse, that is AOT use.

The lift-off mode compiles the wasm binary multiple times, progressively generating a more optimised executable with each iteration. Right now there will only be two iterations. Lift-off mode requires a hand-off process that can be a bit costly. This mode is useful for JIT scenarios where you need the module to start as fast as possible with deferred optimization.

We are using OrcV2 because it promises concurrent compilation and makes JITing a lot easier with support for loading and dumping object code. It also opens up the opportunity of profile-guided optimization in the future.

https://v8.dev/blog/liftoff

##

### LINKING

The following section is now stale because the new design changes idealizes a module as library with some `instantiate` symbol that can be called and passed a `store_addr`. This allows us to compile a module into a library that is reusable across process as it can be loaded and initialized.

When a wasm module is compiled, the internal functions and globals are resolved to their final addresses. But runtime context-dependent objects are not.

Context-dependent objects include:

1. Imported functions
2. Imported globals
3. All memories
4. All tables

These are resolved at instantiation time. Wasmo solves this borrowing ideas from PIC implementation using PLT and GOT.

One thing to note is that traditional linking is different from wasm module instantiation because the order in which external call are made is reversed.

In traditional linking, the in-memory executable asks for an external symbol (in a shared library) to be resolved, so the shared library is loaded into memory on as needed basis.
The shared library doesn't have to know anything about the in-memory executable.

In wasm's case however, you can think of the in-memory executables as the runtime context. Runtime context refers to imported functions, imported globals, memories and tables instantiated by the host. These objects are dynamic. Memories and globals for example cannot be shared between processes. On the other hand, the compiled module becomes the _shared library_ in this situation because it can be shared between different runtime contexts. But unlike traditional linking, the module (the _shared library_) calls the runtime context. This makes typical PIC and load-time relocation strategies unapplicable. It needs to modified.

##

### RESOLUTION

The following section is now stale because the new design changes allow function resolver to be generated within the llvm module making a reserved function resolver address unnecessary.

For wasmo, we generate a `function_resolver` function and a _special data section_ for every module. These properties are inaccessible to internal WebAssembly objects.
The `function_resolver` function is called during instantiation and it sets the address of the external resolver function in the special data section.

![diagram](media/resolution.png)

##

### PROPOSED API

```rs
pub extern "C" hello() -> {
    println!("Hello, world!");
}

let hello_func = Function::new(
    FuncType::new(params: &[], returns: &[]),
    hello as *const ()
);

let memory = Memory::new(1, Some(1));

let imports = &Imports {
    functions: vec![("custom", "hello", hello_func)],
    memories: vec![("custom", "memory", memory)],
    Default::default(),
};

let store = Store::new(Refs::Imports(imports)); // There is also Refs::Objects

let module = Module::new(&bytes, Options::default())?;

// Creates a derived store internally to include builtins and local memories, tables, etc.
// let objects = &Objects {
//     functions: vec![builtin_func, ...],
//     memories: vec![local_memory, ...],
// }
// let store = Store::derive(&store, Refs::Objects(objects))?;
let instance = Instance::new(&module, &store)?;
```

##

### EMBEDDING

The actual implementation is going to be a variation of this:

```rs
// Represents a store for all the external things a wasm instance can access.
pub trait Store {
    // Creates an empty store.
    pub fn init() -> Store;
}

// Represents an uninstantiated module.
pub trait Module {
    // Instantiates a module using the given store and imports addresses.
    pub fn instantiate(&mut self, store: &Store, imports: &[ExternVal]) => (Store, Result<Instance>);

    // Validates the module.
    pub fn validate(module: &Module) -> Result<()>;

    // Returns the module's imports and their names.
    pub fn imports(module: &Module) -> Vec<(String, String, ExternType)>;

    // Returns the module's exports and their names.
    pub fn exports(module: &Module) -> Result<(String, ExternType)>;

    // Decodes a wasm binary into a Module.
    pub fn decode(wasm_binary: &[u8]) -> Result<Module>;

    // Parses a wasm text format into a Module.
    pub fn parse(wasm_text: &str) -> RESULT<Module>;
}

// Represents and instantiated module.
pub trait Instance {
    // Returns the instance's exports addresses.
    pub fn export(module: &Module, name: &str) -> Result<ExternVal>;
}

// Represents an importable function.
pub trait Function {
    // Creates a function with the given signature and address.
    pub fn alloc(store: &Store, sig: &FuncType, host_addr: &HostFunc) -> (Store, FuncAddr);

    // Gets the type of the function from the store.
    pub fn type(store: &Store, addr: &FuncAddr) -> FuncType;

    // Calls the function with the given arguments.
    pub fn invoke(store: &Store, addr: &FuncAddr, &[Val]) -> (Store, Result<Vec<Val>>);
}

pub trait Table {
    // Creates a table with the given type.
    pub fn alloc(store: &Store, ty: &TableType) -> (Store, TableAddr, Ref);

    // Gets the type of the table from the store.
    pub fn type(store: &Store, addr: &TableAddr) -> TableType;

    // Reads an element from the table.
    pub fn read(store: &Store, addr: &TableAddr, index: u32) -> Result<Ref>;

    // Writes an element to the table.
    pub fn write(store: &Store, addr: &TableAddr, index: u32, val: &Ref) -> Result<Store>;

    // Gets the table's size.
    pub fn size(store: &Store, addr: &TableAddr) -> u32;

    // Grows the table.
    pub fn grow(store: &Store, addr: &TableAddr, by: u32) -> Result<Store>;
}

pub trait Memory {
    // Creates a memory with the given type.
    pub fn alloc(store: &Store, ty: &MemoryType) -> (Store, MemoryAddr);

    // Gets the type of the memory from the store.
    pub fn type(store: &Store, addr: &MemoryAddr) -> MemoryType;

    // Reads a byte from the memory.
    pub fn read(store: &Store, addr: &MemoryAddr, offset: u32) -> Result<u8>;

    // Writes a byte to the memory.
    pub fn write(store: &Store, addr: &MemoryAddr, offset: u32, val: u8) -> Result<Store>;

    // Gets the memory's size.
    pub fn size(store: &Store, addr: &MemoryAddr) -> u32;

    // Grows the memory.
    pub fn grow(store: &Store, addr: &MemoryAddr, by: u32) -> Result<Store>;
}

// Represents a global.
pub trait Global {
    // Creates a global with the given type.
    pub fn alloc(store: &Store, ty: &GlobalType, val: Val) -> (Store, GlobalAddr);

    // Gets the type of the global from the store.
    pub fn type(store: &Store, addr: &GlobalAddr) -> GlobalType;

    // Reads the global.
    pub fn read(store: &Store, addr: &GlobalAddr) -> Val;

    // Writes the global.
    pub fn write(store: &Store, addr: &GlobalAddr, val: Val) -> Result<Store>;
}
```

https://webassembly.github.io/spec/core/appendix/embedding.html
