#!/bin/bash
# Test script for CLI argument functionality

echo "=== Testing Ralph CLI Argument Feature ==="
echo ""

# Test 1: Valid project path
echo "Test 1: Launch with valid project path"
echo "Command: ./src-tauri/target/debug/ralph4days --project /tmp/test-ralph-project"
echo "(This should launch the app with the project locked)"
echo ""

# Test 2: Invalid project path
echo "Test 2: Launch with invalid project path"
echo "Command: ./src-tauri/target/debug/ralph4days --project /tmp/nonexistent"
echo "(This should show an error and exit)"
echo ""

# Test 3: No CLI arg
echo "Test 3: Launch without CLI argument"
echo "Command: ./src-tauri/target/debug/ralph4days"
echo "(This should show the ProjectPicker modal)"
echo ""

echo "=== Run these commands manually to test ==="
