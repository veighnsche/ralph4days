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

# Start development server with a mock project (skips project picker)
dev-mock FIXTURE:
    #!/usr/bin/env bash
    if [ ! -d "mock/{{FIXTURE}}" ]; then
        echo "Mock directory not found. Creating from fixtures..."
        just reset-mock
    fi
    bun tauri dev -- -- --project {{justfile_directory()}}/mock/{{FIXTURE}}

# Run cargo check (fast compilation check)
check:
    cargo check --manifest-path src-tauri/Cargo.toml

# Run lints (Rust + TypeScript)
lint:
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
    oxlint src
    bun exec biome lint src

# Fix linting issues automatically
lint-fix:
    bun exec biome lint --write src

# Format all code (Rust + TypeScript)
fmt:
    cargo fmt --manifest-path src-tauri/Cargo.toml
    bun exec biome format --write src

# Check formatting without writing
fmt-check:
    cargo fmt --manifest-path src-tauri/Cargo.toml --check
    bun exec biome format src

# Run all checks (lint + format)
check-all: lint fmt-check

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

# === Mock Test Data ===

# Reset mock directory from fixtures (copies fixtures → mock, makes .ralph visible)
reset-mock:
    @bash scripts/reset-mock.sh

# List available mock projects
list-mock:
    #!/usr/bin/env bash
    if [ ! -d "mock" ]; then
        echo "No mock directory found. Run 'just reset-mock' first."
        exit 1
    fi
    echo "Available mock projects:"
    for f in mock/*/; do
        name=$(basename "$f")
        prd="${f}.ralph/prd.yaml"
        metadata="${f}.ralph/db/metadata.yaml"
        tasks_file="${f}.ralph/db/tasks.yaml"
        if [ -f "$metadata" ]; then
            title=$(grep "^  title:" "$metadata" | cut -d'"' -f2 || echo "N/A")
            tasks=$(grep -c "^  - id:" "$tasks_file" 2>/dev/null || echo "0")
            echo "  $name: $tasks tasks - $title [db format]"
        elif [ -f "$prd" ]; then
            title=$(grep "^  title:" "$prd" | cut -d'"' -f2 || echo "N/A")
            tasks=$(grep -c "^  - id:" "$prd" || echo "0")
            echo "  $name: $tasks tasks - $title [prd.yaml]"
        fi
    done

# === Fixtures (Read-only reference data) ===

# List available fixtures (note: use mock/ for testing)
list-fixtures:
    #!/usr/bin/env bash
    echo "Available fixtures (read-only, use 'just reset-mock' for testing):"
    for f in fixtures/*/; do
        name=$(basename "$f")
        prd="${f}.undetect-ralph/prd.yaml"
        metadata="${f}.undetect-ralph/db/metadata.yaml"
        tasks_file="${f}.undetect-ralph/db/tasks.yaml"
        if [ -f "$metadata" ]; then
            title=$(grep "^  title:" "$metadata" | cut -d'"' -f2 || echo "N/A")
            tasks=$(grep -c "^  - id:" "$tasks_file" 2>/dev/null || echo "0")
            echo "  $name: $tasks tasks - $title [db format]"
        elif [ -f "$prd" ]; then
            title=$(grep "^  title:" "$prd" | cut -d'"' -f2 || echo "N/A")
            tasks=$(grep -c "^  - id:" "$prd" || echo "0")
            echo "  $name: $tasks tasks - $title [prd.yaml]"
        fi
    done
