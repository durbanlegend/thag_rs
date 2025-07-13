#!/bin/bash

# thag_demo - One-line install and demo script
# This script installs thag_demo and runs a quick demonstration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${CYAN}$1${NC}"
}

# Check if cargo is installed
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust first:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
}

# Check if thag_demo is already installed
check_existing_installation() {
    if command -v thag_demo &> /dev/null; then
        print_warning "thag_demo is already installed"
        local current_version=$(thag_demo --version 2>/dev/null | cut -d' ' -f2 || echo "unknown")
        echo "Current version: $current_version"
        read -p "Do you want to reinstall? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "Skipping installation, running demo with existing version"
            return 1
        fi
    fi
    return 0
}

# Install thag_demo
install_thag_demo() {
    print_status "Installing thag_demo..."

    # Use --force to overwrite existing installation if needed
    if cargo install --force thag_demo; then
        print_status "✅ thag_demo installed successfully!"
    else
        print_error "Failed to install thag_demo"
        print_status "You can try installing manually with:"
        echo "  cargo install thag_demo"
        exit 1
    fi
}

# Run the demo
run_demo() {
    print_header ""
    print_header "🔥 Welcome to thag_demo!"
    print_header "========================"
    echo

    # Show available demos
    print_status "Available demos:"
    thag_demo --list

    echo
    print_status "Running basic profiling demo..."
    echo

    # Run the basic profiling demo
    if thag_demo basic-profiling; then
        echo
        print_status "✅ Demo completed successfully!"
        echo
        print_header "🎯 What's Next?"
        print_header "==============="
        echo
        echo "Try more demos:"
        echo "  thag_demo memory-profiling    # Memory allocation tracking"
        echo "  thag_demo async-profiling     # Async function profiling"
        echo "  thag_demo flamegraph          # Interactive flamegraph generation"
        echo "  thag_demo benchmark           # Comprehensive benchmark"
        echo
        echo "Explore generated files:"
        echo "  ls -la *.svg *.folded         # Profile data and flamegraphs"
        echo "  open *.svg                    # View flamegraphs in browser"
        echo
        echo "Learn more:"
        echo "  thag_demo --help              # Show all options"
        echo "  thag_demo script <name>       # Run specific demo script"
        echo
        echo "Resources:"
        echo "  📚 thag_profiler docs: https://docs.rs/thag_profiler"
        echo "  🔧 thag_rs repository: https://github.com/durbanlegend/thag_rs"
        echo "  📖 More examples: https://github.com/durbanlegend/thag_rs/tree/main/demo"
        echo
    else
        print_error "Demo failed to run"
        exit 1
    fi
}

# Main execution
main() {
    print_header "🚀 thag_demo Quick Install & Demo"
    print_header "==================================="
    echo

    # Check prerequisites
    check_cargo

    # Check for existing installation
    if check_existing_installation; then
        install_thag_demo
    fi

    # Run the demo
    run_demo
}

# Handle interrupts gracefully
trap 'print_error "Installation interrupted"; exit 1' INT TERM

# Run main function
main "$@"
