# Ralph Loop development commands
# Run with: just <command>
# Install just: cargo install just

set shell := ["bash", "-cu"]

# Default recipe: show available commands
default:
    @just --list

# === Development ===

# Start development server (frontend + backend hot reload)
dev:
    pnpm tauri dev

# Start frontend dev server only
dev-frontend:
    pnpm dev

# Run cargo check (fast compilation check)
check:
    cargo check --manifest-path src-tauri/Cargo.toml

# Run clippy lints
lint:
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Format all code
fmt:
    cargo fmt --manifest-path src-tauri/Cargo.toml
    pnpm exec prettier --write "src/**/*.{ts,tsx}"

# === Testing ===

# Run all tests
test: test-rust test-frontend

# Run Rust tests
test-rust:
    cargo test --manifest-path src-tauri/Cargo.toml

# Run frontend unit tests
test-frontend:
    pnpm test:run

# Run E2E tests (requires built frontend)
test-e2e:
    pnpm test:e2e

# Run visual regression tests
test-visual:
    pnpm test:visual

# Run chaos/monkey tests
test-monkey:
    pnpm test:monkey

# Update visual test snapshots
test-visual-update:
    pnpm exec playwright test e2e/visual/ --update-snapshots

# === Building ===

# Build release binary (optimized for Alder Lake)
build:
    pnpm tauri build

# Build debug binary (faster compilation)
build-debug:
    pnpm tauri build --debug

# Build frontend only
build-frontend:
    pnpm build

# Clean build artifacts
clean:
    cargo clean --manifest-path src-tauri/Cargo.toml
    rm -rf dist/

# === Release ===

# Build all Linux packages (deb, rpm, appimage)
release-linux:
    pnpm tauri build --bundles deb,rpm,appimage

# === Utilities ===

# Install Playwright browsers
playwright-install:
    pnpm exec playwright install --with-deps

# Check if mold linker is installed
check-mold:
    @which mold > /dev/null && echo "✓ mold linker installed" || echo "✗ mold not found - install with: sudo dnf install mold"

# Show system info relevant to development
sysinfo:
    @echo "=== CPU ==="
    @lscpu | grep "Model name"
    @echo "\n=== Memory ==="
    @free -h | head -2
    @echo "\n=== GPU ==="
    @nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>/dev/null || echo "No NVIDIA GPU"
    @echo "\n=== Rust ==="
    @rustc --version
    @echo "\n=== Node ==="
    @node --version

# Open project in VS Code
code:
    code .

# Watch for file changes and run tests
watch-test:
    cargo watch --manifest-path src-tauri/Cargo.toml -x test

# Generate TypeScript types from Rust (if using ts-rs)
types:
    @echo "TODO: Add ts-rs type generation"
