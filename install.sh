#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Emoji for better UX
CHECK="✓"
CROSS="✗"
ROCKET="🚀"
FOX="🦊"

echo ""
echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                                        ║${NC}"
echo -e "${CYAN}║   ${FOX}  graxaim installer ${FOX}            ║${NC}"
echo -e "${CYAN}║                                        ║${NC}"
echo -e "${CYAN}║   Smart .env profile management        ║${NC}"
echo -e "${CYAN}║                                        ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════╝${NC}"
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to add cargo to PATH for current session
add_cargo_to_path() {
    export PATH="$HOME/.cargo/bin:$PATH"
}

# Check if Rust is installed
echo -e "${BLUE}[1/4]${NC} Checking for Rust..."
if command_exists rustc && command_exists cargo; then
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    echo -e "${GREEN}${CHECK}${NC} Rust ${RUST_VERSION} is installed"
else
    echo -e "${YELLOW}⚠${NC}  Rust is not installed"
    echo ""
    echo -e "${CYAN}Would you like to install Rust now? (required for graxaim)${NC}"
    read -p "Install Rust? [Y/n] " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
        echo -e "${BLUE}${ROCKET}${NC} Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

        # Source cargo env
        source "$HOME/.cargo/env"
        add_cargo_to_path

        echo -e "${GREEN}${CHECK}${NC} Rust installed successfully!"
    else
        echo -e "${RED}${CROSS}${NC} Rust is required to install graxaim"
        echo -e "${YELLOW}Install Rust from: https://rustup.rs${NC}"
        exit 1
    fi
fi

# Ensure cargo is in PATH
add_cargo_to_path

# Check if we're in the graxaim directory
echo ""
echo -e "${BLUE}[2/4]${NC} Checking source files..."

if [ -f "Cargo.toml" ] && grep -q "name = \"graxaim\"" Cargo.toml 2>/dev/null; then
    echo -e "${GREEN}${CHECK}${NC} Found graxaim source in current directory"
    GRAXAIM_DIR="."
else
    echo -e "${YELLOW}⚠${NC}  Not in graxaim directory"

    # Check if user wants to clone from git
    if command_exists git; then
        echo ""
        echo -e "${CYAN}Would you like to clone the graxaim repository?${NC}"
        read -p "Clone repository? [Y/n] " -n 1 -r
        echo ""

        if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
            CLONE_DIR="${HOME}/graxaim"
            echo -e "${BLUE}${ROCKET}${NC} Cloning to ${CLONE_DIR}..."
            git clone https://github.com/gabriel-taufer/graxaim.git "$CLONE_DIR" || {
                echo -e "${RED}${CROSS}${NC} Failed to clone repository"
                echo -e "${YELLOW}Please clone manually or run this script from the graxaim directory${NC}"
                exit 1
            }
            GRAXAIM_DIR="$CLONE_DIR"
        else
            echo -e "${RED}${CROSS}${NC} Please run this script from the graxaim source directory"
            exit 1
        fi
    else
        echo -e "${RED}${CROSS}${NC} Please run this script from the graxaim source directory"
        exit 1
    fi
fi

cd "$GRAXAIM_DIR"

# Build and install
echo ""
echo -e "${BLUE}[3/4]${NC} Building graxaim..."
echo -e "${CYAN}This may take a few minutes...${NC}"
echo ""

if cargo build --release; then
    echo ""
    echo -e "${GREEN}${CHECK}${NC} Build successful!"
else
    echo ""
    echo -e "${RED}${CROSS}${NC} Build failed"
    echo -e "${YELLOW}Check the error messages above for details${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}[4/4]${NC} Installing graxaim..."

if cargo install --path . --force; then
    echo -e "${GREEN}${CHECK}${NC} Installation successful!"
else
    echo ""
    echo -e "${RED}${CROSS}${NC} Installation failed"
    exit 1
fi

# Verify installation
echo ""
echo -e "${BLUE}Verifying installation...${NC}"

add_cargo_to_path

if command_exists graxaim; then
    GRAXAIM_VERSION=$(graxaim --version 2>/dev/null || echo "graxaim")
    echo -e "${GREEN}${CHECK}${NC} graxaim is ready! (${GRAXAIM_VERSION})"

    # Check if cargo bin is in PATH
    if ! echo "$PATH" | grep -q "$HOME/.cargo/bin"; then
        echo ""
        echo -e "${YELLOW}⚠${NC}  Add Rust to your PATH by running:"
        echo ""

        # Detect shell
        if [ -n "$ZSH_VERSION" ]; then
            echo -e "    ${CYAN}echo 'export PATH=\"\$HOME/.cargo/bin:\$PATH\"' >> ~/.zshrc${NC}"
            echo -e "    ${CYAN}source ~/.zshrc${NC}"
        elif [ -n "$BASH_VERSION" ]; then
            echo -e "    ${CYAN}echo 'export PATH=\"\$HOME/.cargo/bin:\$PATH\"' >> ~/.bashrc${NC}"
            echo -e "    ${CYAN}source ~/.bashrc${NC}"
        else
            echo -e "    ${CYAN}export PATH=\"\$HOME/.cargo/bin:\$PATH\"${NC}"
        fi
        echo ""
        echo -e "${YELLOW}Or start a new terminal session${NC}"
    fi
else
    echo -e "${YELLOW}⚠${NC}  graxaim installed but not found in PATH"
    echo -e "${YELLOW}Try starting a new terminal session${NC}"
fi

# Success message with next steps
echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                                        ║${NC}"
echo -e "${GREEN}║  ${FOX}  Installation Complete! ${FOX}         ║${NC}"
echo -e "${GREEN}║                                        ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "${CYAN}Quick start:${NC}"
echo ""
echo -e "  ${BLUE}1.${NC} Go to your project:"
echo -e "     ${CYAN}cd ~/my-project${NC}"
echo ""
echo -e "  ${BLUE}2.${NC} Initialize graxaim:"
echo -e "     ${CYAN}graxaim init${NC}"
echo ""
echo -e "  ${BLUE}3.${NC} Create and switch profiles:"
echo -e "     ${CYAN}graxaim create local${NC}"
echo -e "     ${CYAN}graxaim use local${NC}"
echo ""
echo -e "${CYAN}Learn more:${NC}"
echo -e "  ${CYAN}graxaim --help${NC}"
echo -e "  ${CYAN}graxaim use --help${NC}"
echo ""
echo -e "${GREEN}Happy profile switching! ${FOX}${NC}"
echo ""
