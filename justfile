_default:
    @just --list

# Format code
fmt:
    cargo fmt

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Type-check without building
check:
    cargo check

# Run unit tests
test:
    cargo test

# Build in debug mode
build:
    cargo build

# Build release binary
release:
    cargo build --release

# Format + lint + test + build (CI-style)
ci: fmt lint test build

# Remove build artifacts
clean:
    cargo clean

# Run the full pipeline
run:
    cargo run

# Full quality check: format, lint, test, check
qa: fmt lint test check
