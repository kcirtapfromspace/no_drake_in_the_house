#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

echo "🛠️  Installing Modern Kubernetes Development Tools"
echo "================================================"

# Detect OS
OS="unknown"
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
fi

print_status "Detected OS: $OS"

# Check if Homebrew is available (macOS)
if [[ "$OS" == "macos" ]]; then
    if ! command -v brew &> /dev/null; then
        print_error "Homebrew is not installed. Please install it first:"
        echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        exit 1
    fi
fi

# Install tools based on OS
install_tool() {
    local tool=$1
    local macos_cmd=$2
    local linux_cmd=$3
    
    if command -v "$tool" &> /dev/null; then
        print_success "$tool is already installed"
        return
    fi
    
    print_status "Installing $tool..."
    
    if [[ "$OS" == "macos" ]]; then
        eval "$macos_cmd"
    elif [[ "$OS" == "linux" ]]; then
        eval "$linux_cmd"
    else
        print_error "Unsupported OS for automatic installation"
        return 1
    fi
    
    if command -v "$tool" &> /dev/null; then
        print_success "$tool installed successfully"
    else
        print_error "Failed to install $tool"
    fi
}

# Install Docker (if not present)
if ! command -v docker &> /dev/null; then
    print_warning "Docker is not installed. Please install Docker Desktop manually:"
    if [[ "$OS" == "macos" ]]; then
        echo "  Download from: https://www.docker.com/products/docker-desktop"
        echo "  Or use: brew install --cask docker"
    else
        echo "  Follow instructions at: https://docs.docker.com/engine/install/"
    fi
else
    print_success "Docker is already installed"
fi

# Install kubectl
install_tool "kubectl" \
    "brew install kubectl" \
    "curl -LO \"https://dl.k8s.io/release/\$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl\" && chmod +x kubectl && sudo mv kubectl /usr/local/bin/"

# Install minikube
install_tool "minikube" \
    "brew install minikube" \
    "curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64 && sudo install minikube-linux-amd64 /usr/local/bin/minikube"

# Install Helm
install_tool "helm" \
    "brew install helm" \
    "curl https://get.helm.sh/helm-v3.13.0-linux-amd64.tar.gz | tar xz && sudo mv linux-amd64/helm /usr/local/bin/"

# Install Tilt
install_tool "tilt" \
    "brew install tilt-dev/tap/tilt" \
    "curl -fsSL https://raw.githubusercontent.com/tilt-dev/tilt/master/scripts/install.sh | bash"

# Install Skaffold
install_tool "skaffold" \
    "brew install skaffold" \
    "curl -Lo skaffold https://storage.googleapis.com/skaffold/releases/latest/skaffold-linux-amd64 && sudo install skaffold /usr/local/bin/"

echo ""
print_success "🎉 Installation complete!"
echo ""
echo "📋 Installed tools:"
echo "   kubectl: $(kubectl version --client --short 2>/dev/null || echo 'Not found')"
echo "   minikube: $(minikube version --short 2>/dev/null || echo 'Not found')"
echo "   helm: $(helm version --short 2>/dev/null || echo 'Not found')"
echo "   tilt: $(tilt version 2>/dev/null || echo 'Not found')"
echo "   skaffold: $(skaffold version --output=json 2>/dev/null | grep -o '"version":"[^"]*' | cut -d'"' -f4 || echo 'Not found')"
echo ""
echo "🚀 Next steps:"
echo "   1. Start minikube: minikube start --driver=docker --memory=4096 --cpus=2"
echo "   2. Start development: make k8s-dev"
echo "   3. Open Tilt UI: http://localhost:10350"
echo ""
echo "📖 Read the full guide: KUBERNETES_DEVELOPMENT.md"