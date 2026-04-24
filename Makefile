# RustyTeams — common build tasks.
#
# Cross-platform. Works from:
#   - Windows: git-bash / MSYS / WSL (bash required on PATH)
#   - Linux:   any POSIX shell with GNU Make
#
# The Rust toolchain and cargo must be on PATH. `make setup` bootstraps the
# CEF binaries for the host platform.

# --- platform detection -----------------------------------------------------

ifeq ($(OS),Windows_NT)
  PLATFORM        := windows
  EXE_SUFFIX      := .exe
  HOME_DIR        := $(USERPROFILE)
else
  UNAME_S         := $(shell uname -s)
  ifeq ($(UNAME_S),Linux)
    PLATFORM      := linux
    EXE_SUFFIX    :=
    HOME_DIR      := $(HOME)
  else
    $(error Unsupported platform: $(UNAME_S). RustyTeams supports Windows and Linux)
  endif
endif

# --- shell selection --------------------------------------------------------
# Force bash everywhere. On Windows, GNU Make defaults to cmd.exe — override
# so our recipes, quoting, and pipefail behave the same as on Linux. Git for
# Windows puts bash.exe on PATH (inside `C:\Program Files\Git\bin`).

ifeq ($(PLATFORM),windows)
  ifneq ($(shell bash -c "echo ok" 2>&1),ok)
    $(error bash not found on PATH. Install Git for Windows and re-open \
your shell, or invoke this Makefile from a Git Bash prompt)
  endif
endif
SHELL := bash
.SHELLFLAGS := -eu -o pipefail -c

# --- paths ------------------------------------------------------------------

# Where the CEF Standard Distribution lives. Override by exporting CEF_PATH or
# passing `make CEF_PATH=... <target>`.
CEF_PATH     ?= $(HOME_DIR)/.local/share/cef
CEF_RS_REPO  ?= https://github.com/tauri-apps/cef-rs
CEF_RS_DIR   ?= .cache/cef-rs

TARGET_DIR   ?= target
DEBUG_DIR    := $(TARGET_DIR)/debug
RELEASE_DIR  := $(TARGET_DIR)/release
DIST_DIR     := dist
APP_NAME     := rustyteams
EXE          := $(APP_NAME)$(EXE_SUFFIX)

CARGO        ?= cargo

# cargo's build.rs for the `cef` crate reads CEF_PATH to locate libcef's import
# library (Windows: libcef.dll.lib, Linux: libcef.so). Runtime shared-library
# loading is handled by `_sync-runtime-*`, which copies the CEF binaries next
# to the exe — no PATH / LD_LIBRARY_PATH munging needed.
export CEF_PATH

# --- platform-specific CEF runtime files ------------------------------------
# What we copy next to the exe so CEF can resolve resources at launch.

ifeq ($(PLATFORM),windows)
  CEF_LIB           := libcef.dll
  CEF_RUNTIME_FILES := \
    libcef.dll chrome_elf.dll \
    libEGL.dll libGLESv2.dll d3dcompiler_47.dll \
    dxcompiler.dll dxil.dll \
    vk_swiftshader.dll vk_swiftshader_icd.json vulkan-1.dll \
    icudtl.dat \
    v8_context_snapshot.bin \
    chrome_100_percent.pak chrome_200_percent.pak resources.pak
else
  CEF_LIB           := libcef.so
  CEF_RUNTIME_FILES := \
    libcef.so \
    libEGL.so libGLESv2.so \
    libvk_swiftshader.so vk_swiftshader_icd.json libvulkan.so.1 \
    chrome-sandbox \
    icudtl.dat \
    v8_context_snapshot.bin snapshot_blob.bin \
    chrome_100_percent.pak chrome_200_percent.pak resources.pak
endif

# --- phony targets ----------------------------------------------------------

.PHONY: help setup build release run run-release check fmt fmt-check \
        clippy test clean clean-all package dist installer deb tarball \
        doctor print-env \
        _sync-runtime-debug _sync-runtime-release

help: ## Show this help.
	@echo "RustyTeams make targets (platform=$(PLATFORM)):"
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
	  awk -F':.*?## ' '{printf "  %-14s %s\n", $$1, $$2}'

# --- bootstrap --------------------------------------------------------------

setup: $(CEF_PATH)/$(CEF_LIB) ## Fetch CEF Standard Distribution to $CEF_PATH.

$(CEF_PATH)/$(CEF_LIB):
	@mkdir -p $(dir $(CEF_RS_DIR))
	@if [ ! -d "$(CEF_RS_DIR)" ]; then \
	  echo ">> cloning $(CEF_RS_REPO) into $(CEF_RS_DIR)"; \
	  git clone --depth 1 $(CEF_RS_REPO) $(CEF_RS_DIR); \
	fi
	@echo ">> exporting CEF binaries to $(CEF_PATH)"
	cd $(CEF_RS_DIR) && $(CARGO) run -p export-cef-dir -- --force "$(CEF_PATH)"

# --- build ------------------------------------------------------------------

build: ## Debug build (syncs CEF runtime next to the exe).
	$(CARGO) build
	@$(MAKE) --no-print-directory _sync-runtime-debug

release: ## Release build (syncs CEF runtime next to the exe).
	$(CARGO) build --release
	@$(MAKE) --no-print-directory _sync-runtime-release

check: ## Type-check without producing binaries.
	$(CARGO) check --all-targets

# --- run --------------------------------------------------------------------

run: build ## Build debug and launch.
	./$(DEBUG_DIR)/$(EXE)

run-release: release ## Build release and launch.
	./$(RELEASE_DIR)/$(EXE)

# Copy libcef + ICU + V8 snapshots + .pak files + locales/ next to the binary.
# CEF resolves all these via its own executable directory at runtime.
# `export-cef-dir` flattens the Spotify distribution into a single tree, so we
# copy straight from $CEF_PATH.

_sync-runtime-debug: | $(DEBUG_DIR)
	@echo ">> syncing CEF runtime -> $(DEBUG_DIR)"
	@for f in $(CEF_RUNTIME_FILES); do \
	  if [ -f "$(CEF_PATH)/$$f" ]; then cp -u "$(CEF_PATH)/$$f" "$(DEBUG_DIR)/"; fi; \
	done
	@if [ -d "$(CEF_PATH)/locales" ]; then cp -Ru "$(CEF_PATH)/locales" "$(DEBUG_DIR)/"; fi
ifeq ($(PLATFORM),linux)
	@# chrome-sandbox must be setuid-root to be usable; if root is available
	@# chown+chmod, otherwise leave the file and rely on --no-sandbox at runtime.
	@if [ -f "$(DEBUG_DIR)/chrome-sandbox" ] && [ "$$(id -u)" = "0" ]; then \
	  chown root:root "$(DEBUG_DIR)/chrome-sandbox"; \
	  chmod 4755 "$(DEBUG_DIR)/chrome-sandbox"; \
	fi
endif

_sync-runtime-release: | $(RELEASE_DIR)
	@echo ">> syncing CEF runtime -> $(RELEASE_DIR)"
	@for f in $(CEF_RUNTIME_FILES); do \
	  if [ -f "$(CEF_PATH)/$$f" ]; then cp -u "$(CEF_PATH)/$$f" "$(RELEASE_DIR)/"; fi; \
	done
	@if [ -d "$(CEF_PATH)/locales" ]; then cp -Ru "$(CEF_PATH)/locales" "$(RELEASE_DIR)/"; fi
ifeq ($(PLATFORM),linux)
	@if [ -f "$(RELEASE_DIR)/chrome-sandbox" ] && [ "$$(id -u)" = "0" ]; then \
	  chown root:root "$(RELEASE_DIR)/chrome-sandbox"; \
	  chmod 4755 "$(RELEASE_DIR)/chrome-sandbox"; \
	fi
endif

$(DEBUG_DIR) $(RELEASE_DIR):
	@mkdir -p $@

# --- hygiene ----------------------------------------------------------------

fmt: ## Format all Rust sources.
	$(CARGO) fmt --all

fmt-check: ## Verify formatting (CI-friendly).
	$(CARGO) fmt --all -- --check

clippy: ## Lint with clippy (warnings become errors).
	$(CARGO) clippy --all-targets -- -D warnings

test: ## Run unit tests.
	$(CARGO) test

# --- packaging --------------------------------------------------------------

package: release ## Stage a runnable folder at dist/rustyteams/.
	@rm -rf $(DIST_DIR)/$(APP_NAME)
	@mkdir -p $(DIST_DIR)/$(APP_NAME)
	@cp -R $(RELEASE_DIR)/. $(DIST_DIR)/$(APP_NAME)/
	@echo ">> packaged to $(DIST_DIR)/$(APP_NAME)/"

# `dist` produces a user-distributable archive in the native format:
#   - Windows: .zip (Compress-Archive / zip fallback)
#   - Linux:   .tar.gz
ifeq ($(PLATFORM),windows)
dist: package ## Produce dist/rustyteams.zip (Windows) / .tar.gz (Linux).
	@cd $(DIST_DIR) && rm -f $(APP_NAME).zip && \
	  (powershell -NoProfile -Command "Compress-Archive -Path '$(APP_NAME)/*' -DestinationPath '$(APP_NAME).zip' -Force" \
	   || zip -r $(APP_NAME).zip $(APP_NAME))
	@echo ">> $(DIST_DIR)/$(APP_NAME).zip"
else
dist: tarball

tarball: package ## Linux: produce dist/rustyteams.tar.gz.
	@cd $(DIST_DIR) && rm -f $(APP_NAME).tar.gz && \
	  tar -czf $(APP_NAME).tar.gz $(APP_NAME)
	@echo ">> $(DIST_DIR)/$(APP_NAME).tar.gz"
endif

# `installer` produces a platform-native installable:
#   - Windows: NSIS .exe installer
#   - Linux:   .deb via cargo-deb
ifeq ($(PLATFORM),windows)
installer: package ## Build platform installer (NSIS on Windows, .deb on Linux).
	@command -v makensis >/dev/null 2>&1 || { \
	  echo "makensis not found. Install from https://nsis.sourceforge.io/ or 'winget install NSIS.NSIS'"; \
	  exit 1; \
	}
	makensis -V2 installer.nsi
	@echo ">> installer built in $(DIST_DIR)/"
else
installer: deb

deb: release ## Linux: build a .deb via cargo-deb (see [package.metadata.deb] in Cargo.toml).
	@command -v cargo-deb >/dev/null 2>&1 || { \
	  echo "cargo-deb not found. Install with 'cargo install cargo-deb'"; \
	  exit 1; \
	}
	@mkdir -p $(DIST_DIR)
	$(CARGO) deb --no-build --output $(DIST_DIR)/
	@echo ">> .deb built in $(DIST_DIR)/"
endif

# --- cleanup ----------------------------------------------------------------

clean: ## Remove target/ and dist/.
	$(CARGO) clean
	@rm -rf $(DIST_DIR)

clean-all: clean ## Also remove cached CEF binaries and the cef-rs checkout.
	@rm -rf $(CEF_RS_DIR)
	@echo ">> (keeping $(CEF_PATH) -- delete manually if you want to re-download CEF)"

# --- diagnostics ------------------------------------------------------------

doctor: ## Sanity-check the environment before building.
	@FAIL=0; \
	echo "platform: $(PLATFORM)"; \
	printf "cargo   : "; $(CARGO) --version || FAIL=1; \
	printf "rustc   : "; rustc --version || FAIL=1; \
	printf "cmake   : "; cmake --version 2>/dev/null | head -n1 || { echo "NOT FOUND"; FAIL=1; }; \
	printf "ninja   : "; ninja --version 2>/dev/null || echo "NOT FOUND (optional on Linux)"; \
	if [ "$(PLATFORM)" = "windows" ]; then \
	  printf "link.exe: "; command -v link.exe >/dev/null 2>&1 && echo "on PATH (good — MSVC toolchain active)" || echo "NOT ON PATH (run from VS Developer prompt, or let cargo find MSVC)"; \
	else \
	  printf "cc      : "; command -v cc >/dev/null 2>&1 && cc --version | head -n1 || { echo "NOT FOUND (install build-essential)"; FAIL=1; }; \
	  printf "pkgcfg  : "; command -v pkg-config >/dev/null 2>&1 && pkg-config --version || echo "NOT FOUND (install pkg-config)"; \
	fi; \
	printf "CEF_PATH: %s\n" "$(CEF_PATH)"; \
	if [ -f "$(CEF_PATH)/$(CEF_LIB)" ]; then \
	  printf "libcef  : found (%s)\n" "$(CEF_PATH)/$(CEF_LIB)"; \
	else \
	  printf "libcef  : NOT FOUND at %s\n" "$(CEF_PATH)/$(CEF_LIB)"; \
	  if [ -d "$(CEF_PATH)" ]; then \
	    printf "          CEF_PATH exists. Contents:\n"; \
	    ls -1 "$(CEF_PATH)" | sed 's/^/            /'; \
	  else \
	    printf "          CEF_PATH does not exist. Run 'make setup'.\n"; \
	  fi; \
	  FAIL=1; \
	fi; \
	exit $$FAIL

print-env: ## Print the effective CEF_PATH / PATH for debugging.
	@echo "PLATFORM=$(PLATFORM)"
	@echo "CEF_PATH=$(CEF_PATH)"
	@echo "PATH=$(PATH)"
