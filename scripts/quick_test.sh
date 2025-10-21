#!/bin/bash

# =============================================================================
# Market SaaS Backend - Quick Test Runner
# =============================================================================
# This script runs essential tests quickly for development workflow
# =============================================================================

set -e

# Disable parallel testing for compatibility
export RUST_TEST_THREADS=1

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo -e "\n${BLUE}=== $1 ===${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Quick test execution
main() {
    print_header "Quick Test Suite"
    
    local start_time=$(date +%s)
    local failed=false
    
    # Format check
    print_info "Checking code formatting..."
    if cargo fmt --check; then
        print_success "Code formatting OK"
    else
        print_error "Code formatting issues found"
        failed=true
    fi
    
    # Clippy check
    print_info "Running clippy..."
    if cargo clippy --quiet -- -D warnings; then
        print_success "Clippy checks passed"
    else
        print_error "Clippy found issues"
        failed=true
    fi
    
    # Unit tests
    print_info "Running unit tests (single-threaded)..."
    if cargo test --lib --quiet -- --test-threads=1; then
        print_success "Unit tests passed"
    else
        print_error "Unit tests failed"
        failed=true
    fi
    
    # Integration tests
    print_info "Running integration tests (single-threaded)..."
    if cargo test --test '*' --quiet -- --test-threads=1; then
        print_success "Integration tests passed"
    else
        print_error "Integration tests failed"
        failed=true
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    print_info "Execution time: ${duration}s"
    
    if [ "$failed" = true ]; then
        print_error "Some tests failed!"
        exit 1
    else
        print_success "All tests passed! 🎉"
        exit 0
    fi
}

main "$@" 