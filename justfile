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
    bun tauri dev

# Start frontend dev server only
dev-frontend:
    bun dev

# Start development server with a specific project (skips project picker)
dev-with-project PROJECT:
    bun tauri dev -- --project {{PROJECT}}

# Start development server with single-task fixture
dev-single-task:
    bun tauri dev -- --project {{justfile_directory()}}/fixtures/single-task

# Run cargo check (fast compilation check)
check:
    cargo check --manifest-path src-tauri/Cargo.toml

# Run clippy lints
lint:
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Format all code
fmt:
    cargo fmt --manifest-path src-tauri/Cargo.toml
    bun exec prettier --write "src/**/*.{ts,tsx}"

# === Testing ===

# Run all tests
test: test-rust test-frontend

# Run Rust tests
test-rust:
    cargo test --manifest-path src-tauri/Cargo.toml

# Run frontend unit tests
test-frontend:
    bun test:run

# Run E2E tests (requires built frontend)
test-e2e:
    bun test:e2e

# Run visual regression tests
test-visual:
    bun test:visual

# Run chaos/monkey tests
test-monkey:
    bun test:monkey

# Update visual test snapshots
test-visual-update:
    bun exec playwright test e2e/visual/ --update-snapshots

# === Building ===

# Build release binary (optimized for Alder Lake)
build:
    bun tauri build

# Build debug binary (faster compilation)
build-debug:
    bun tauri build --debug

# Build frontend only
build-frontend:
    bun build

# Clean build artifacts
clean:
    cargo clean --manifest-path src-tauri/Cargo.toml
    rm -rf dist/

# === Release ===

# Build all Linux packages (deb, rpm, appimage)
release-linux:
    bun tauri build --bundles deb,rpm,appimage

# === Utilities ===

# Install Playwright browsers
playwright-install:
    bun exec playwright install --with-deps

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

# === Fixtures ===

# Reset all fixtures to initial state
reset-fixtures:
    @echo "Resetting all fixtures..."
    @for fixture in fixtures/*/reset.sh; do \
        if [ -f "$$fixture" ]; then \
            echo "  Resetting $$(dirname $$fixture)..."; \
            bash "$$fixture" > /dev/null; \
        fi; \
    done
    @echo "✓ All fixtures reset"

# Reset single-task fixture
reset-single-task:
    @bash fixtures/single-task/reset.sh

# List available fixtures
list-fixtures:
    #!/usr/bin/env bash
    echo "Available fixtures:"
    for f in fixtures/*/; do
        name=$(basename "$f")
        prd="${f}.ralph/prd.yaml"
        if [ -f "$prd" ]; then
            title=$(grep "^  title:" "$prd" | cut -d'"' -f2 || echo "N/A")
            tasks=$(grep -c "^  - id:" "$prd" || echo "0")
            echo "  $name: $tasks tasks - $title"
        fi
    done

# Clean fixture generated files (but don't reset PRD status)
clean-fixtures:
    @echo "Cleaning fixture outputs..."
    @rm -f fixtures/*/.ralph/progress.txt fixtures/*/.ralph/learnings.txt
    @rm -f fixtures/*/CLAUDE.md fixtures/*/CLAUDE.md.ralph-backup
    @echo "✓ Generated files removed"
