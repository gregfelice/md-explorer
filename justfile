# md-explorer task runner

# Default recipe - show available commands
default:
    @just --list

# Build debug version
build:
    cargo build

# Build release version
release:
    cargo build --release

# Run tests
test:
    cargo test

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without changing
fmt-check:
    cargo fmt -- --check

# Install to ~/.cargo/bin
install:
    cargo install --path .

# Uninstall
uninstall:
    cargo uninstall md-explorer

# Run the application
run:
    cargo run

# Clean build artifacts
clean:
    cargo clean

# Full check (fmt, lint, test)
check: fmt-check lint test

# Prepare for release (check + build release)
prep: check release
