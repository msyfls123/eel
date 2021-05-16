## Run

```shell
cargo wasi build
wasmtime ./target/wasm32-wasi/debug/eel.rustc.wasm --mapdir ./::./ ./ target
node --experimental-wasi-unstable-preview1 index.js . src
```
