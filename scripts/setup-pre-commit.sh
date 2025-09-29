#!/bin/bash

# Setup script for pre-commit hooks
# This script installs and configures pre-commit hooks for the project

set -euo pipefail

echo "🔧 Setting up pre-commit hooks for No Drake in the House..."

# Check if we're in the project root
if [[ ! -f ".pre-commit-config.yaml" ]]; then
    echo "❌ Error: .pre-commit-config.yaml not found. Please run this script from the project root."
    exit 1
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install pre-commit if not available
if ! command_exists pre-commit; then
    echo "📦 Installing pre-commit..."
    
    if command_exists pip; then
        pip install pre-commit
    elif command_exists pip3; then
        pip3 install pre-commit
    elif command_exists brew; then
        brew install pre-commit
    else
        echo "❌ Error: Could not install pre-commit. Please install it manually:"
        echo "   pip install pre-commit"
        echo "   or visit: https://pre-commit.com/#installation"
        exit 1
    fi
fi

# Install the git hook scripts
echo "🔗 Installing pre-commit git hooks..."
pre-commit install

# Install commit-msg hook for conventional commits (optional)
if command_exists commitizen || command_exists cz; then
    echo "📝 Installing commit-msg hook for conventional commits..."
    pre-commit install --hook-type commit-msg
fi

# Create secrets baseline if it doesn't exist
if [[ ! -f ".secrets.baseline" ]]; then
    echo "🔒 Creating secrets baseline..."
    if command_exists detect-secrets; then
        detect-secrets scan --baseline .secrets.baseline
    else
        echo "⚠️  Warning: detect-secrets not found. Installing..."
        pip install detect-secrets
        detect-secrets scan --baseline .secrets.baseline
    fi
fi

# Install additional tools if not present
echo "🛠️  Checking for additional development tools..."

# Check for Rust tools
if command_exists cargo; then
    echo "✅ Rust toolchain found"
    
    # Install rustfmt and clippy if not available
    if ! rustup component list --installed | grep -q rustfmt; then
        echo "📦 Installing rustfmt..."
        rustup component add rustfmt
    fi
    
    if ! rustup component list --installed | grep -q clippy; then
        echo "📦 Installing clippy..."
        rustup component add clippy
    fi
else
    echo "⚠️  Warning: Rust toolchain not found. Please install Rust:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi

# Check for Node.js and npm
if command_exists node && command_exists npm; then
    echo "✅ Node.js toolchain found"
    
    # Install frontend dependencies if package.json exists
    if [[ -f "frontend/package.json" ]]; then
        echo "📦 Installing frontend dependencies..."
        cd frontend
        npm install
        cd ..
    fi
else
    echo "⚠️  Warning: Node.js not found. Please install Node.js 18+:"
    echo "   https://nodejs.org/"
fi

# Check for optional tools
echo "🔍 Checking for optional development tools..."

optional_tools=(
    "hadolint:Dockerfile linting"
    "shellcheck:Shell script linting"
    "kubeval:Kubernetes manifest validation"
    "sqlfluff:SQL formatting and linting"
)

for tool_info in "${optional_tools[@]}"; do
    tool="${tool_info%%:*}"
    description="${tool_info##*:}"
    
    if command_exists "$tool"; then
        echo "✅ $tool found ($description)"
    else
        echo "⚠️  Optional: $tool not found ($description)"
        case "$tool" in
            "hadolint")
                echo "   Install: brew install hadolint (macOS) or see https://github.com/hadolint/hadolint"
                ;;
            "shellcheck")
                echo "   Install: brew install shellcheck (macOS) or apt install shellcheck (Ubuntu)"
                ;;
            "kubeval")
                echo "   Install: brew install kubeval (macOS) or see https://github.com/instrumenta/kubeval"
                ;;
            "sqlfluff")
                echo "   Install: pip install sqlfluff"
                ;;
        esac
    fi
done

# Run pre-commit on all files to test setup
echo "🧪 Testing pre-commit setup..."
if pre-commit run --all-files; then
    echo "✅ Pre-commit hooks setup successfully!"
else
    echo "⚠️  Some pre-commit checks failed. This is normal for the first run."
    echo "   The hooks will automatically fix many issues."
    echo "   Run 'pre-commit run --all-files' again to see remaining issues."
fi

echo ""
echo "🎉 Pre-commit setup complete!"
echo ""
echo "📋 What happens now:"
echo "   • Pre-commit hooks will run automatically on 'git commit'"
echo "   • Code will be formatted and linted before each commit"
echo "   • Security checks will scan for secrets and vulnerabilities"
echo ""
echo "🔧 Useful commands:"
echo "   pre-commit run --all-files    # Run hooks on all files"
echo "   pre-commit run <hook-name>    # Run specific hook"
echo "   pre-commit autoupdate         # Update hook versions"
echo "   git commit --no-verify        # Skip hooks (not recommended)"
echo ""
echo "📚 For more info: https://pre-commit.com/"