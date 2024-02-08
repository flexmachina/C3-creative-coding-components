# Build & Run

There are a few different ways to build to app.

## WebXR wasm

1. Build the wasm:
```
> make wasm
```

2. Run a webserver serving this directory. Use `-p` to change the port.
```
> ./webserver.py
```

3. Launch a web browser at `localhost:8000`

The Meta Quest browser requires https for non localhost addresses, but you can workaround this by setting up a reverse socket
connection with adb:

```
adb reverse tcp:8000 tcp:8000
```

This allows you to browse to `localhost:8000` on the Quest Browser and access the webserver running on your development machine.

## Normal wasm

For debugging purposes. Shares most of the same rendering and wasm code.

1. Set `XR_MODE = false` in `src/lib.rs`
2. Follow the same steps as above.

## Desktop

For debugging purposes. Shares most of the same rendering code.

```
> cargo run
```
