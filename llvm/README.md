Rust bindings for LLVM specifically written for WebAssembly frontends.

## Safety Notes

This library is not guaranteed to be safe. It requires more testing and sanitization for confidence.

It is not an easy task mapping the ownership semantics of LLVM ffi items over to Rust. Attempting to do so results in a really tedious API.

So this library relies on the fact that LLVM modules and contexts are not thread safe meaning we don't have to worry about lifetimes of ffi items shared between threads. They are always !Send.

Also based on the ownership information gotten from the docs, we have designed the API to be safe to an extent. For example, a module cannot be created without a context. This is not ideal in some cases but it is the trade-off we have had to make.

## Consideration

There are a lot of self referencing problems and unclear ownership in the LLVM API. We could Rc everything and use pinning for the self referential issues but it doesn't completely solve our problem.
Sometimes we need to expose a wrapped pointer back to the underlying LLVM API.

A proper solution would require clear understanding of ownerships in LLVM as well as maintaining ownership vectors in the Rust side of the API. Clearly, this will lead to duplication of logic and a heavy wrapper.
