[private]
@default: help

# show help message
@help:
    echo "Usage: just <recipe>"
    echo ""
    just --list

dev:
    cargo run --features bevy/dynamic_linking
