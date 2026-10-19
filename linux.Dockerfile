FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    file \
    libayatana-appindicator3-dev \
    libgtk-3-dev \
    librsvg2-dev \
    libssl-dev \
    libwebkit2gtk-4.1-dev \
    libxdo-dev \
    patchelf \
    pkg-config \
    wget \
    xz-utils \
  && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://nodejs.org/dist/v20.19.0/node-v20.19.0-linux-x64.tar.xz \
    | tar -xJ -C /usr/local --strip-components=1

RUN curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /src
COPY . .
RUN npm install
RUN apt-get update && apt-get install -y --no-install-recommends xdg-utils \
  && rm -rf /var/lib/apt/lists/*
RUN npm run tauri -w @confluity/desktop -- build --bundles deb,appimage
RUN mkdir -p /out && cp -a apps/desktop/src-tauri/target/release/bundle /out/bundle
