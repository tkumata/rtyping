FROM mcr.microsoft.com/vscode/devcontainers/rust:latest

RUN sudo apt-get update -y && sudo apt-get install g++ pkg-config libasound2-dev libudev-dev -y
RUN rustup component add rustfmt clippy
