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
    # Headless Chromium for browser checks with Playwright (npx playwright
    # install chromium downloads the browser itself; these are its libraries).
    libnss3 \
    libnspr4 \
    libdbus-1-3 \
    libatk1.0-0t64 \
    libatk-bridge2.0-0t64 \
    libatspi2.0-0t64 \
    libcups2t64 \
    libxcomposite1 \
    libxdamage1 \
    libasound2t64 \
    libgbm1 \
    libpango-1.0-0 \
    libcairo2 \
    # pkg-config: build dependency of wayland-sys (sketch_ui via eframe/glutin)
    pkg-config \
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

# aider: delegates small, test-verified coding tasks to the host's Ollama
# (see tools/local-coder/). Installed system-wide so the vscode user can run it.
ENV UV_TOOL_DIR=/opt/uv-tools \
    UV_TOOL_BIN_DIR=/usr/local/bin \
    UV_PYTHON_INSTALL_DIR=/opt/uv-python
RUN curl -LsSf https://astral.sh/uv/install.sh | env UV_INSTALL_DIR=/usr/local/bin sh \
  && uv tool install --python 3.12 aider-chat \
  && aider --version

# Claude Code CLI. Its config/credentials live in a named volume
# (see devcontainer.json) so login survives container rebuilds.
ENV CLAUDE_CONFIG_DIR=/home/vscode/.claude
RUN npm install -g @anthropic-ai/claude-code \
  && claude --version \
  && mkdir -p /home/vscode/.claude \
  && chown vscode:vscode /home/vscode/.claude
