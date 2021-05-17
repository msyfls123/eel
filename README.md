## Run

```shell
cargo wasi build
wasmtime ./target/wasm32-wasi/debug/eel.wasi.wasm --mapdir .::. . target CACHEDIR.TAG
node --experimental-wasi-unstable-preview1 index.js . src main.rs
```
