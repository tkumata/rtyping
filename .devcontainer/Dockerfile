FROM mcr.microsoft.com/vscode/devcontainers/rust:latest

RUN sudo apt update -y && export DEBIAN_FRONTEND=noninteractive sudo apt install g++ pkg-config libasound2-dev libudev-dev alsa alsa-utils -y
RUN rustup component add rustfmt clippy
