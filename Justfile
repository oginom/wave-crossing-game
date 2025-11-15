[private]
@default: help

# show help message
@help:
    echo "Usage: just <recipe>"
    echo ""
    just --list

dev:
    cargo run --features bevy/dynamic_linking

# Build for WebGL/WASM (optimized for size)
build-wasm:
    cargo build --profile wasm-release --target wasm32-unknown-unknown
    wasm-bindgen --out-dir ./wasm \
      --out-name wave_crossing_game \
      --target web \
      ./target/wasm32-unknown-unknown/wasm-release/wave_crossing_game.wasm
    @echo "Copying assets..."
    @rm -rf ./wasm/assets
    @cp -r ./assets ./wasm/assets

# Build for WebGL/WASM (debug, faster build)
build-wasm-dev:
    cargo build --target wasm32-unknown-unknown
    wasm-bindgen --out-dir ./wasm \
      --out-name wave_crossing_game \
      --target web \
      ./target/wasm32-unknown-unknown/debug/wave_crossing_game.wasm
    @echo "Copying assets..."
    @rm -rf ./wasm/assets
    @cp -r ./assets ./wasm/assets

# Build and serve WASM build locally
serve-wasm: build-wasm
    @echo "Starting web server at http://localhost:8000"
    @echo "Press Ctrl+C to stop"
    cd wasm && python3 -m http.server 8000
