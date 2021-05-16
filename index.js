const fs = require('fs')
const path = require('path')
const { pathToFileURL } = require('url')
const { WASI } = require('wasi')

console.log(__dirname, path.resolve(), process.cwd())

const preopen = process.argv[2]

console.log(preopen)

const wasi = new WASI({
  args: process.argv.slice(1),
  env: process.env,
  preopens: {
    '.': __dirname,
    '/sandbox': __dirname,
  }
})

const importObject = {
  wasi_snapshot_preview1: wasi.wasiImport,
};

(async () => {
  const wasm = await WebAssembly.compile(fs.readFileSync('./target/wasm32-wasi/debug/eel.wasi.wasm'))
  const instance = await WebAssembly.instantiate(wasm, importObject)

  wasi.start(instance)
})()
