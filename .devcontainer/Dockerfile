FROM mcr.microsoft.com/devcontainers/rust:1-trixie

# Tooling previously provided by Dev Container features (git, gh, node)
# plus X11/OpenGL libs for egui/winit (host display via /tmp/.X11-unix).
RUN apt-get update \
  && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    gnupg \
    git \
    libxcursor1 \
    libxrandr2 \
    libxi6 \
    libx11-6 \
    libxkbcommon0 \
    libxkbcommon-x11-0 \
    libxcb1 \
    libxcb-render0 \
    libxcb-shape0 \
    libxcb-xfixes0 \
    libgl1 \
    libegl1 \
    libfontconfig1 \
  # GitHub CLI
  && curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
    | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
  && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
  && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
    > /etc/apt/sources.list.d/github-cli.list \
  # Node.js LTS (for OpenSpec CLI)
  && curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
  && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    gh \
    nodejs \
  && npm install -g @fission-ai/openspec@latest \
  && openspec --version \
  && node --version \
  && gh --version \
  && git --version \
  && rm -rf /var/lib/apt/lists/*
