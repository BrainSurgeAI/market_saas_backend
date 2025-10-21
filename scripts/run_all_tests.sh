#!/bin/bash

# =============================================================================
# Market SaaS Backend - Comprehensive Test Runner Script
# =============================================================================
# This script runs all types of tests for the Market SaaS Backend project
# including unit tests, integration tests, documentation tests, and more.
# =============================================================================

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="market-saas-backend"
COVERAGE_THRESHOLD=80
TEST_TIMEOUT=300  # 5 minutes
CARGO_FLAGS="--color=always"

# Disable parallel testing for compatibility
export RUST_TEST_THREADS=1

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Functions
print_header() {
    echo -e "\n${BLUE}============================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}============================================${NC}\n"
}

print_section() {
    echo -e "\n${CYAN}--- $1 ---${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${PURPLE}ℹ $1${NC}"
}

# Check if required tools are installed
check_dependencies() {
    print_section "Checking Dependencies"
    
    local missing_deps=()
    
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo")
    fi
    
    if ! command -v rustc &> /dev/null; then
        missing_deps+=("rustc")
    fi
    
    # Check for optional tools
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_warning "cargo-tarpaulin not found. Code coverage will be skipped."
        print_info "Install with: cargo install cargo-tarpaulin"
    fi
    
    if ! command -v cargo-audit &> /dev/null; then
        print_warning "cargo-audit not found. Security audit will be skipped."
        print_info "Install with: cargo install cargo-audit"
    fi
    
    if ! command -v cargo-outdated &> /dev/null; then
        print_warning "cargo-outdated not found. Dependency check will be skipped."
        print_info "Install with: cargo install cargo-outdated"
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
    
    print_success "All required dependencies are available"
}

# Clean previous build artifacts
clean_build() {
    print_section "Cleaning Build Artifacts"
    
    if cargo clean; then
        print_success "Build artifacts cleaned"
    else
        print_error "Failed to clean build artifacts"
        exit 1
    fi
}

# Check code formatting
check_formatting() {
    print_section "Checking Code Formatting"
    
    if cargo fmt --check; then
        print_success "Code formatting is correct"
        ((PASSED_TESTS++))
    else
        print_error "Code formatting issues found. Run 'cargo fmt' to fix."
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
}

# Run clippy lints
run_clippy() {
    print_section "Running Clippy Lints"
    
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_success "Clippy lints passed"
        ((PASSED_TESTS++))
    else
        print_error "Clippy lints failed"
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
}

# Run unit tests
run_unit_tests() {
    print_section "Running Unit Tests"
    
    print_info "Running all unit tests (single-threaded)..."
    if timeout $TEST_TIMEOUT cargo test $CARGO_FLAGS --lib -- --test-threads=1; then
        print_success "Unit tests passed"
        ((PASSED_TESTS++))
    else
        print_error "Unit tests failed"
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
}

# Run integration tests
run_integration_tests() {
    print_section "Running Integration Tests"
    
    print_info "Running integration tests (single-threaded)..."
    if timeout $TEST_TIMEOUT cargo test $CARGO_FLAGS --test '*' -- --test-threads=1; then
        print_success "Integration tests passed"
        ((PASSED_TESTS++))
    else
        print_error "Integration tests failed"
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
}

# Run documentation tests
run_doc_tests() {
    print_section "Running Documentation Tests"
    
    print_info "Running documentation tests (single-threaded)..."
    if timeout $TEST_TIMEOUT cargo test $CARGO_FLAGS --doc -- --test-threads=1; then
        print_success "Documentation tests passed"
        ((PASSED_TESTS++))
    else
        print_error "Documentation tests failed"
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
}

# Run specific module tests
run_module_tests() {
    print_section "Running Module-Specific Tests"
    
    local modules=(
        "permission"
        "auth"
        "tenant"
        "user"
        "order"
        "delivery"
        "market"
        "provider"
    )
    
    for module in "${modules[@]}"; do
        print_info "Testing module: $module"
        if cargo test $CARGO_FLAGS $module --lib -- --test-threads=1; then
            print_success "Module $module tests passed"
        else
            print_warning "Module $module tests failed or not found"
        fi
    done
}

# Run tests with different feature flags
run_feature_tests() {
    print_section "Running Feature Flag Tests"
    
    print_info "Testing with no default features..."
    if cargo test $CARGO_FLAGS --no-default-features -- --test-threads=1; then
        print_success "No default features tests passed"
        ((PASSED_TESTS++))
    else
        print_error "No default features tests failed"
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
    
    print_info "Testing with all features..."
    if cargo test $CARGO_FLAGS --all-features -- --test-threads=1; then
        print_success "All features tests passed"
        ((PASSED_TESTS++))
    else
        print_error "All features tests failed"
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
}

# Run performance tests
run_performance_tests() {
    print_section "Running Performance Tests"
    
    print_info "Running performance benchmarks (single-threaded)..."
    if cargo test $CARGO_FLAGS --release -- --ignored --test-threads=1; then
        print_success "Performance tests passed"
        ((PASSED_TESTS++))
    else
        print_warning "Performance tests failed or not found"
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
}

# Generate code coverage report
generate_coverage() {
    print_section "Generating Code Coverage Report"
    
    if command -v cargo-tarpaulin &> /dev/null; then
        print_info "Generating coverage report with tarpaulin..."
        
        if cargo tarpaulin --out Html --output-dir target/coverage --timeout $TEST_TIMEOUT; then
            local coverage=$(cargo tarpaulin --out Stdout | grep -oP '\d+\.\d+(?=% coverage)' | tail -1)
            
            if [ -n "$coverage" ]; then
                if (( $(echo "$coverage >= $COVERAGE_THRESHOLD" | bc -l) )); then
                    print_success "Code coverage: $coverage% (above threshold: $COVERAGE_THRESHOLD%)"
                    ((PASSED_TESTS++))
                else
                    print_warning "Code coverage: $coverage% (below threshold: $COVERAGE_THRESHOLD%)"
                    ((FAILED_TESTS++))
                fi
            else
                print_warning "Could not determine coverage percentage"
                ((FAILED_TESTS++))
            fi
            
            print_info "Coverage report generated at: target/coverage/tarpaulin-report.html"
        else
            print_error "Failed to generate coverage report"
            ((FAILED_TESTS++))
        fi
        ((TOTAL_TESTS++))
    else
        print_warning "Skipping coverage report (cargo-tarpaulin not installed)"
    fi
}

# Run security audit
run_security_audit() {
    print_section "Running Security Audit"
    
    if command -v cargo-audit &> /dev/null; then
        print_info "Running security audit..."
        if cargo audit; then
            print_success "Security audit passed"
            ((PASSED_TESTS++))
        else
            print_error "Security audit found vulnerabilities"
            ((FAILED_TESTS++))
        fi
        ((TOTAL_TESTS++))
    else
        print_warning "Skipping security audit (cargo-audit not installed)"
    fi
}

# Check for outdated dependencies
check_outdated_deps() {
    print_section "Checking for Outdated Dependencies"
    
    if command -v cargo-outdated &> /dev/null; then
        print_info "Checking for outdated dependencies..."
        if cargo outdated --exit-code 1; then
            print_success "All dependencies are up to date"
            ((PASSED_TESTS++))
        else
            print_warning "Some dependencies are outdated"
            ((FAILED_TESTS++))
        fi
        ((TOTAL_TESTS++))
    else
        print_warning "Skipping outdated dependency check (cargo-outdated not installed)"
    fi
}

# Validate Cargo.toml
validate_cargo_toml() {
    print_section "Validating Cargo.toml"
    
    print_info "Checking Cargo.toml syntax..."
    if cargo check --quiet; then
        print_success "Cargo.toml is valid"
        ((PASSED_TESTS++))
    else
        print_error "Cargo.toml validation failed"
        ((FAILED_TESTS++))
    fi
    ((TOTAL_TESTS++))
}

# Run database migration tests (if applicable)
run_migration_tests() {
    print_section "Running Database Migration Tests"
    
    if [ -d "migrations" ]; then
        print_info "Testing database migrations..."
        # Add specific migration test logic here if needed
        print_warning "Migration tests not implemented yet"
    else
        print_info "No migrations directory found, skipping migration tests"
    fi
}

# Generate test report
generate_test_report() {
    print_section "Test Report"
    
    local success_rate=0
    if [ $TOTAL_TESTS -gt 0 ]; then
        success_rate=$(( (PASSED_TESTS * 100) / TOTAL_TESTS ))
    fi
    
    echo -e "Total Tests: ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
    echo -e "Success Rate: ${BLUE}$success_rate%${NC}"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        print_success "All tests completed successfully! 🎉"
        return 0
    else
        print_error "Some tests failed. Please review the output above."
        return 1
    fi
}

# Main execution
main() {
    print_header "Market SaaS Backend - Comprehensive Test Suite"
    
    local start_time=$(date +%s)
    
    # Parse command line arguments
    local run_coverage=true
    local run_security=true
    local run_performance=false
    local clean_first=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-coverage)
                run_coverage=false
                shift
                ;;
            --no-security)
                run_security=false
                shift
                ;;
            --performance)
                run_performance=true
                shift
                ;;
            --clean)
                clean_first=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --no-coverage    Skip code coverage generation"
                echo "  --no-security    Skip security audit"
                echo "  --performance    Include performance tests"
                echo "  --clean          Clean build artifacts first"
                echo "  --help, -h       Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Run test suite
    check_dependencies
    
    if [ "$clean_first" = true ]; then
        clean_build
    fi
    
    validate_cargo_toml
    check_formatting
    run_clippy
    run_unit_tests
    run_integration_tests
    run_doc_tests
    run_module_tests
    run_feature_tests
    
    if [ "$run_performance" = true ]; then
        run_performance_tests
    fi
    
    if [ "$run_coverage" = true ]; then
        generate_coverage
    fi
    
    if [ "$run_security" = true ]; then
        run_security_audit
        check_outdated_deps
    fi
    
    run_migration_tests
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    print_info "Total execution time: ${duration}s"
    
    generate_test_report
}

# Execute main function with all arguments
main "$@" 