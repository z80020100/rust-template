#!/bin/bash

CARGO_CMD="cargo"
DOCKER_CMD="docker"
CROSS_CMD="cross"

CROSS_REPO_URL="https://github.com/cross-rs/cross.git"
CUSTOM_CROSS_REPO_URL="https://github.com/z80020100/cross.git"
CUSTOM_CROSS_BRANCH="aarch64_host_platform_custom_image"

# Install dependencies
function install_dependencies() {
  sudo apt-get update
  sudo apt-get install -y \
    build-essential \
    curl \
    git
}

# Install Rust
function install_rust() {
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  . $HOME/.cargo/env
}

# Install Docker
function install_docker() {
  # Set up the repository
  sudo apt-get update
  sudo apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release
  curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list >/dev/null

  # Install Docker Engine
  sudo apt-get update
  sudo apt-get install -y docker-ce docker-ce-cli containerd.io

  # Manage Docker as a non-root user
  sudo gpasswd -a $USER docker
}

# Install cross
function install_cross() {
  cargo install cross --git $CROSS_REPO_URL
}

# Install custom cross
function install_custom_cross() {
  echo "Install custom cross from $CUSTOM_CROSS_BRANCH branch of $CUSTOM_CROSS_REPO_URL"
  cargo install cross --git $CUSTOM_CROSS_REPO_URL --branch $CUSTOM_CROSS_BRANCH
}

function main() {
  install_dependencies
  required_cmds=(
    "$CARGO_CMD"
    "$DOCKER_CMD"
    "$CROSS_CMD"
  )
  for cmd in "${required_cmds[@]}"; do
    if command -v "$cmd" &>/dev/null; then
      echo "$cmd is installed"
    else
      echo "Installing $cmd..."
      case $cmd in
      "$CARGO_CMD")
        install_rust
        ;;
      "$DOCKER_CMD")
        install_docker
        ;;
      "$CROSS_CMD")
        install_custom_cross
        ;;
      esac
    fi
  done
}

main
