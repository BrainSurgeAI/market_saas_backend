#!/bin/bash

# =============================================================================
# Market SaaS Backend - Clippy Fix Helper
# =============================================================================
# This script helps fix common clippy warnings automatically where possible
# =============================================================================

set -e

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

# Apply automatic fixes where possible
apply_auto_fixes() {
    print_header "Applying Automatic Clippy Fixes"
    
    print_info "Running cargo clippy --fix..."
    if cargo clippy --fix --allow-dirty --allow-staged; then
        print_success "Automatic fixes applied"
    else
        print_error "Some fixes could not be applied automatically"
    fi
}

# Show remaining issues
show_remaining_issues() {
    print_header "Checking Remaining Issues"
    
    print_info "Running clippy to check remaining issues..."
    cargo clippy --all-targets --all-features -- -D warnings || true
}

# Suggest manual fixes
suggest_manual_fixes() {
    print_header "Manual Fix Suggestions"
    
    echo "The following issues typically need manual fixes:"
    echo ""
    echo "1. Too many arguments (>7) - Consider using a struct"
    echo "2. Empty lines after doc comments - Remove extra blank lines"
    echo "3. Direct ToString implementation - Implement Display instead"
    echo "4. Missing Default implementation - Add #[derive(Default)] or impl Default"
    echo "5. Should implement trait - Use standard traits like FromStr"
    echo ""
    echo "For detailed guidance, run: cargo clippy --all-targets --all-features"
}

main() {
    print_header "Clippy Fix Helper"
    
    # Check if there are uncommitted changes
    if ! git diff --quiet; then
        print_info "You have uncommitted changes. The --allow-dirty flag will be used."
    fi
    
    apply_auto_fixes
    show_remaining_issues
    suggest_manual_fixes
    
    print_info "Run './scripts/quick_test.sh' to verify fixes"
}

main "$@" 