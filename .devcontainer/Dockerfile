FROM mcr.microsoft.com/vscode/devcontainers/rust:latest

RUN sudo apt-get update -y && sudo apt-get install -y g++ pkg-config libasound2 libasound2-dev libudev-dev pulseaudio alsa-utils
RUN rustup component add rustfmt clippy
