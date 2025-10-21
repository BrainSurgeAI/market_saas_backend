#!/bin/bash

# =============================================================================
# Market SaaS Backend - Coverage Report Generator
# =============================================================================
# This script generates detailed test coverage reports
# =============================================================================

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Configuration
COVERAGE_THRESHOLD=${COVERAGE_THRESHOLD:-75}
OUTPUT_DIR="target/coverage"
REPORT_NAME="coverage-report"

print_header() {
    echo -e "\n${BLUE}============================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}============================================${NC}\n"
}

print_section() {
    echo -e "\n${PURPLE}--- $1 ---${NC}\n"
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
    echo -e "${BLUE}ℹ $1${NC}"
}

# Check if tarpaulin is installed
check_tarpaulin() {
    print_section "Checking Dependencies"
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_error "cargo-tarpaulin is not installed"
        print_info "Install with: cargo install cargo-tarpaulin"
        exit 1
    fi
    
    print_success "cargo-tarpaulin is available"
}

# Clean previous coverage data
clean_coverage() {
    print_section "Cleaning Previous Coverage Data"
    
    if [ -d "$OUTPUT_DIR" ]; then
        rm -rf "$OUTPUT_DIR"
        print_success "Previous coverage data cleaned"
    else
        print_info "No previous coverage data found"
    fi
    
    mkdir -p "$OUTPUT_DIR"
}

# Generate coverage report
generate_coverage() {
    print_section "Generating Coverage Report"
    
    local start_time=$(date +%s)
    
    print_info "Running tests with coverage analysis..."
    print_info "This may take a few minutes..."
    
    # Generate multiple output formats
    if cargo tarpaulin \
        --out Html \
        --out Xml \
        --out Json \
        --output-dir "$OUTPUT_DIR" \
        --timeout 300 \
        --verbose; then
        
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        print_success "Coverage report generated in ${duration}s"
    else
        print_error "Failed to generate coverage report"
        exit 1
    fi
}

# Extract and display coverage statistics
display_coverage_stats() {
    print_section "Coverage Statistics"
    
    # Try to extract coverage from JSON output
    local json_file="$OUTPUT_DIR/tarpaulin-report.json"
    local html_file="$OUTPUT_DIR/tarpaulin-report.html"
    local xml_file="$OUTPUT_DIR/cobertura.xml"
    
    if [ -f "$json_file" ]; then
        print_info "Parsing coverage data from JSON report..."
        
        # Extract overall coverage percentage
        local coverage=$(jq -r '.coverage' "$json_file" 2>/dev/null || echo "unknown")
        
        if [ "$coverage" != "unknown" ] && [ "$coverage" != "null" ]; then
            local coverage_percent=$(echo "$coverage * 100" | bc -l | cut -d. -f1)
            
            echo -e "Overall Coverage: ${BLUE}${coverage_percent}%${NC}"
            
            # Check against threshold
            if [ "$coverage_percent" -ge "$COVERAGE_THRESHOLD" ]; then
                print_success "Coverage meets threshold (${COVERAGE_THRESHOLD}%)"
            else
                print_warning "Coverage below threshold (${COVERAGE_THRESHOLD}%)"
            fi
            
            # Extract file-level coverage if available
            if jq -e '.files' "$json_file" >/dev/null 2>&1; then
                echo -e "\nFile Coverage Details:"
                jq -r '.files[] | "\(.name): \((.coverage * 100) | floor)%"' "$json_file" | head -20
                
                local total_files=$(jq '.files | length' "$json_file")
                if [ "$total_files" -gt 20 ]; then
                    echo "... and $((total_files - 20)) more files"
                fi
            fi
        else
            print_warning "Could not extract coverage percentage from JSON"
        fi
    else
        print_warning "JSON coverage report not found"
        
        # Fallback: try to extract from stdout
        print_info "Attempting to extract coverage from command output..."
        if cargo tarpaulin --out Stdout | grep -oP '\d+\.\d+(?=% coverage)' | tail -1; then
            print_info "Coverage extracted from stdout"
        else
            print_warning "Could not determine coverage percentage"
        fi
    fi
}

# Generate summary report
generate_summary() {
    print_section "Generating Summary Report"
    
    local summary_file="$OUTPUT_DIR/coverage-summary.txt"
    
    cat > "$summary_file" << EOF
Coverage Report Summary
======================
Generated: $(date)
Project: Market SaaS Backend
Threshold: ${COVERAGE_THRESHOLD}%

Report Files:
- HTML Report: tarpaulin-report.html
- XML Report: cobertura.xml  
- JSON Report: tarpaulin-report.json

Commands Used:
- cargo tarpaulin --out Html --out Xml --out Json --output-dir $OUTPUT_DIR

Notes:
- Open the HTML report in a browser for detailed line-by-line coverage
- Use the XML report for CI/CD integration
- Use the JSON report for programmatic analysis

EOF
    
    print_success "Summary report generated: $summary_file"
}

# Display report locations
show_report_locations() {
    print_section "Report Locations"
    
    echo "Coverage reports have been generated in: $OUTPUT_DIR"
    echo ""
    
    if [ -f "$OUTPUT_DIR/tarpaulin-report.html" ]; then
        echo -e "${GREEN}HTML Report:${NC} $OUTPUT_DIR/tarpaulin-report.html"
        print_info "Open this file in a web browser for interactive coverage view"
    fi
    
    if [ -f "$OUTPUT_DIR/cobertura.xml" ]; then
        echo -e "${GREEN}XML Report:${NC} $OUTPUT_DIR/cobertura.xml"
        print_info "Use this for CI/CD integration (e.g., GitLab CI, GitHub Actions)"
    fi
    
    if [ -f "$OUTPUT_DIR/tarpaulin-report.json" ]; then
        echo -e "${GREEN}JSON Report:${NC} $OUTPUT_DIR/tarpaulin-report.json"
        print_info "Use this for programmatic analysis"
    fi
    
    if [ -f "$OUTPUT_DIR/coverage-summary.txt" ]; then
        echo -e "${GREEN}Summary:${NC} $OUTPUT_DIR/coverage-summary.txt"
    fi
}

# Open HTML report if requested
open_report() {
    local html_file="$OUTPUT_DIR/tarpaulin-report.html"
    
    if [ -f "$html_file" ] && [ "$1" = "--open" ]; then
        print_info "Opening HTML report..."
        
        # Try different browsers/commands
        if command -v xdg-open &> /dev/null; then
            xdg-open "$html_file"
        elif command -v open &> /dev/null; then
            open "$html_file"
        elif command -v firefox &> /dev/null; then
            firefox "$html_file" &
        elif command -v google-chrome &> /dev/null; then
            google-chrome "$html_file" &
        else
            print_warning "Could not open browser automatically"
            print_info "Please open $html_file manually in your browser"
        fi
    fi
}

# Main execution
main() {
    print_header "Coverage Report Generator"
    
    local open_browser=false
    local clean_first=true
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --open)
                open_browser=true
                shift
                ;;
            --no-clean)
                clean_first=false
                shift
                ;;
            --threshold)
                COVERAGE_THRESHOLD="$2"
                shift 2
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --open           Open HTML report in browser after generation"
                echo "  --no-clean       Don't clean previous coverage data"
                echo "  --threshold N    Set coverage threshold (default: 75)"
                echo "  --help, -h       Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    local start_time=$(date +%s)
    
    check_tarpaulin
    
    if [ "$clean_first" = true ]; then
        clean_coverage
    fi
    
    generate_coverage
    display_coverage_stats
    generate_summary
    show_report_locations
    
    if [ "$open_browser" = true ]; then
        open_report --open
    fi
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    print_success "Coverage analysis completed in ${total_duration}s"
    
    echo ""
    print_info "To view the interactive HTML report:"
    echo "  $0 --open"
    print_info "To set a different threshold:"
    echo "  $0 --threshold 85"
}

# Handle script interruption
trap 'echo -e "\n${RED}Coverage generation interrupted${NC}"' INT TERM

main "$@" 