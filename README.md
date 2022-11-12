<div align="center">
    <a href="#" target="_blank">
        <img src="https://raw.githubusercontent.com/appcypher/wasmo-old/master/media/wasmo.png" alt="Wasmo Logo" width="140" height="140"></img>
    </a>
</div>

<h1 align="center">Wasmo</h1>

`wasmo` is a WebAssembly compiler and runtime. It compiles WebAssembly code to native code with runtime memory, control integrity security as outlined by the WebAssembly spec.

##

### GOALS

In this order.

1. Simple implementation.
2. Single-pass compilation.
3. Serializable.
4. Progressive optimisation.

### Getting Started

### Building Project

### Running Examples

Change the `LLVM_SYS_130_PREFIX` variable to point to your LLVM installation.

```bash
LLVM_SYS_130_PREFIX=/opt/homebrew/opt/llvm cargo run --example module tests/samples/experiment.wat
```
