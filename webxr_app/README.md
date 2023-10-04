# WebXR Demo App

Basic example of a WebXR app using wgpu for rendering.

# Build & Run

There are a few different ways to build to app.

## WebXR wasm

1. Build the wasm:
```
> RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --debug --target web
```

2. Run a webserver serving this directory. Use `-p` to change the port.
```
> ./webserver.py
```

3. Launch a web browser at `localhost:8000`


## Normal wasm

For debugging purposes. Shares most of the same rendering and wasm code.

1. Set `XR_MODE = false` in `src/lib.rs`
2. Follow the same steps as above.

## Desktop

For debugging purposes. Shares most of the same rendering code.

```
> cargo run
```
