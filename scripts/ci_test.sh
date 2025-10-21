#!/bin/bash

# =============================================================================
# Market SaaS Backend - CI/CD Test Runner
# =============================================================================
# This script is designed for CI/CD environments with strict checks
# =============================================================================

set -e

# Exit codes
EXIT_SUCCESS=0
EXIT_FORMAT_FAIL=1
EXIT_LINT_FAIL=2
EXIT_TEST_FAIL=3
EXIT_COVERAGE_FAIL=4
EXIT_SECURITY_FAIL=5

# Colors (only if terminal supports it)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Configuration
COVERAGE_THRESHOLD=${COVERAGE_THRESHOLD:-75}
RUST_BACKTRACE=1
CARGO_TERM_COLOR=always

# Disable parallel testing for compatibility
export RUST_TEST_THREADS=1

print_step() {
    echo -e "${BLUE}[CI] $1${NC}"
}

print_success() {
    echo -e "${GREEN}[SUCCESS] $1${NC}"
}

print_error() {
    echo -e "${RED}[ERROR] $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}[WARNING] $1${NC}"
}

# Check environment
check_environment() {
    print_step "Checking CI environment..."
    
    # Check Rust version
    if command -v rustc &> /dev/null; then
        local rust_version=$(rustc --version)
        print_success "Rust version: $rust_version"
    else
        print_error "Rust not found"
        exit 1
    fi
    
    # Check Cargo version
    if command -v cargo &> /dev/null; then
        local cargo_version=$(cargo --version)
        print_success "Cargo version: $cargo_version"
    else
        print_error "Cargo not found"
        exit 1
    fi
    
    # Set environment variables
    export RUST_BACKTRACE=1
    export CARGO_TERM_COLOR=always
}

# Validate project structure
validate_project() {
    print_step "Validating project structure..."
    
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found"
        exit 1
    fi
    
    if [ ! -d "src" ]; then
        print_error "src directory not found"
        exit 1
    fi
    
    print_success "Project structure is valid"
}

# Check code formatting
check_formatting() {
    print_step "Checking code formatting..."
    
    if cargo fmt --all -- --check; then
        print_success "Code formatting is correct"
    else
        print_error "Code formatting issues found. Run 'cargo fmt' to fix."
        exit $EXIT_FORMAT_FAIL
    fi
}

# Run linting
run_linting() {
    print_step "Running Clippy lints..."
    
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_success "All lints passed"
    else
        print_error "Lint issues found"
        exit $EXIT_LINT_FAIL
    fi
}

# Run tests with detailed output
run_tests() {
    print_step "Running all tests..."
    
    # Unit tests
    print_step "Running unit tests..."
    if cargo test --lib --verbose -- --test-threads=1; then
        print_success "Unit tests passed"
    else
        print_error "Unit tests failed"
        exit $EXIT_TEST_FAIL
    fi
    
    # Integration tests
    print_step "Running integration tests..."
    if cargo test --test '*' --verbose -- --test-threads=1; then
        print_success "Integration tests passed"
    else
        print_error "Integration tests failed"
        exit $EXIT_TEST_FAIL
    fi
    
    # Documentation tests
    print_step "Running documentation tests..."
    if cargo test --doc --verbose -- --test-threads=1; then
        print_success "Documentation tests passed"
    else
        print_error "Documentation tests failed"
        exit $EXIT_TEST_FAIL
    fi
}

# Generate coverage report
generate_coverage() {
    print_step "Generating code coverage..."
    
    if command -v cargo-tarpaulin &> /dev/null; then
        # Generate coverage with XML output for CI systems
        if cargo tarpaulin --out Xml --out Html --output-dir target/coverage --timeout 300; then
            # Extract coverage percentage
            local coverage=$(cargo tarpaulin --out Stdout | grep -oP '\d+\.\d+(?=% coverage)' | tail -1)
            
            if [ -n "$coverage" ]; then
                echo "Coverage: $coverage%"
                
                if (( $(echo "$coverage >= $COVERAGE_THRESHOLD" | bc -l) )); then
                    print_success "Coverage $coverage% meets threshold $COVERAGE_THRESHOLD%"
                else
                    print_error "Coverage $coverage% below threshold $COVERAGE_THRESHOLD%"
                    exit $EXIT_COVERAGE_FAIL
                fi
            else
                print_warning "Could not determine coverage percentage"
            fi
            
            # Output coverage file paths for CI artifacts
            echo "Coverage reports generated:"
            echo "  - target/coverage/cobertura.xml"
            echo "  - target/coverage/tarpaulin-report.html"
        else
            print_error "Failed to generate coverage report"
            exit $EXIT_COVERAGE_FAIL
        fi
    else
        print_warning "cargo-tarpaulin not available, skipping coverage"
    fi
}

# Security audit
run_security_audit() {
    print_step "Running security audit..."
    
    if command -v cargo-audit &> /dev/null; then
        if cargo audit; then
            print_success "Security audit passed"
        else
            print_error "Security vulnerabilities found"
            exit $EXIT_SECURITY_FAIL
        fi
    else
        print_warning "cargo-audit not available, skipping security audit"
    fi
}

# Check for unused dependencies
check_unused_deps() {
    print_step "Checking for unused dependencies..."
    
    if command -v cargo-udeps &> /dev/null; then
        if cargo +nightly udeps; then
            print_success "No unused dependencies found"
        else
            print_warning "Unused dependencies detected"
        fi
    else
        print_warning "cargo-udeps not available, skipping unused dependency check"
    fi
}

# Generate test artifacts
generate_artifacts() {
    print_step "Generating test artifacts..."
    
    # Create artifacts directory
    mkdir -p target/ci-artifacts
    
    # Copy test results
    if [ -f "target/coverage/cobertura.xml" ]; then
        cp target/coverage/cobertura.xml target/ci-artifacts/
    fi
    
    if [ -f "target/coverage/tarpaulin-report.html" ]; then
        cp target/coverage/tarpaulin-report.html target/ci-artifacts/
    fi
    
    # Generate test summary
    cat > target/ci-artifacts/test-summary.txt << EOF
Test Summary
============
Date: $(date)
Rust Version: $(rustc --version)
Cargo Version: $(cargo --version)
Coverage Threshold: $COVERAGE_THRESHOLD%

Test Results:
- Code formatting: PASSED
- Linting: PASSED
- Unit tests: PASSED
- Integration tests: PASSED
- Documentation tests: PASSED
EOF
    
    print_success "Artifacts generated in target/ci-artifacts/"
}

# Main execution
main() {
    echo "============================================"
    echo "Market SaaS Backend - CI/CD Test Suite"
    echo "============================================"
    echo "Started at: $(date)"
    echo ""
    
    local start_time=$(date +%s)
    
    # Parse arguments
    local skip_coverage=false
    local skip_security=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-coverage)
                skip_coverage=true
                shift
                ;;
            --skip-security)
                skip_security=true
                shift
                ;;
            --coverage-threshold)
                COVERAGE_THRESHOLD="$2"
                shift 2
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Run CI pipeline
    check_environment
    validate_project
    check_formatting
    run_linting
    run_tests
    
    if [ "$skip_coverage" = false ]; then
        generate_coverage
    fi
    
    if [ "$skip_security" = false ]; then
        run_security_audit
    fi
    
    check_unused_deps
    generate_artifacts
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo ""
    echo "============================================"
    print_success "All CI checks passed! 🎉"
    echo "Total execution time: ${duration}s"
    echo "Completed at: $(date)"
    echo "============================================"
    
    exit $EXIT_SUCCESS
}

# Trap to ensure cleanup on exit
trap 'echo "CI pipeline interrupted"' INT TERM

main "$@" 