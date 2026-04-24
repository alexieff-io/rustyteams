# RustyTeams — common build tasks.
#
# Works from any POSIX-ish shell on Windows (git-bash, MSYS, WSL). The Rust
# toolchain and cargo must be on PATH; `setup` bootstraps the CEF binaries.

# --- shell selection --------------------------------------------------------
# GNU Make on Windows ships with cmd.exe as the default shell. Force bash so
# our recipes, quoting, and pipefail behave. Git for Windows puts bash.exe on
# PATH (inside `C:\Program Files\Git\bin`).

ifeq ($(OS),Windows_NT)
  # Probe for bash (relying on PATH, no absolute path — that way embedded
  # spaces in "C:\Program Files\Git\..." can't break SHELL parsing).
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
CEF_PATH     ?= $(USERPROFILE)/.local/share/cef
CEF_RS_REPO  ?= https://github.com/tauri-apps/cef-rs
CEF_RS_DIR   ?= .cache/cef-rs

TARGET_DIR   ?= target
DEBUG_DIR    := $(TARGET_DIR)/debug
RELEASE_DIR  := $(TARGET_DIR)/release
DIST_DIR     := dist
APP_NAME     := rustyteams
EXE          := $(APP_NAME).exe

CARGO        ?= cargo

# cargo's build.rs for the `cef` crate reads CEF_PATH to locate libcef.dll.lib.
# Runtime DLL loading is handled by `_sync-runtime-*`, which copies libcef.dll
# next to the exe — no PATH munging needed.
export CEF_PATH

# --- phony targets ----------------------------------------------------------

.PHONY: help setup build release run run-release check fmt fmt-check \
        clippy test clean clean-all package dist doctor print-env

help: ## Show this help.
	@echo "RustyTeams make targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
	  awk -F':.*?## ' '{printf "  %-14s %s\n", $$1, $$2}'

# --- bootstrap --------------------------------------------------------------

setup: $(CEF_PATH)/libcef.dll ## Fetch CEF Standard Distribution to $CEF_PATH.

$(CEF_PATH)/libcef.dll:
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

# Copy libcef.dll + ICU + V8 snapshots + .pak files + locales/ next to the
# binary. CEF resolves all these via its own executable directory at runtime.
# `export-cef-dir` flattens the Spotify distribution into a single tree, so we
# copy straight from $CEF_PATH.
CEF_RUNTIME_FILES := \
  libcef.dll chrome_elf.dll \
  libEGL.dll libGLESv2.dll d3dcompiler_47.dll \
  dxcompiler.dll dxil.dll \
  vk_swiftshader.dll vk_swiftshader_icd.json vulkan-1.dll \
  icudtl.dat \
  v8_context_snapshot.bin \
  chrome_100_percent.pak chrome_200_percent.pak resources.pak

_sync-runtime-debug: | $(DEBUG_DIR)
	@echo ">> syncing CEF runtime -> $(DEBUG_DIR)"
	@for f in $(CEF_RUNTIME_FILES); do \
	  if [ -f "$(CEF_PATH)/$$f" ]; then cp -u "$(CEF_PATH)/$$f" "$(DEBUG_DIR)/"; fi; \
	done
	@if [ -d "$(CEF_PATH)/locales" ]; then cp -Ru "$(CEF_PATH)/locales" "$(DEBUG_DIR)/"; fi

_sync-runtime-release: | $(RELEASE_DIR)
	@echo ">> syncing CEF runtime -> $(RELEASE_DIR)"
	@for f in $(CEF_RUNTIME_FILES); do \
	  if [ -f "$(CEF_PATH)/$$f" ]; then cp -u "$(CEF_PATH)/$$f" "$(RELEASE_DIR)/"; fi; \
	done
	@if [ -d "$(CEF_PATH)/locales" ]; then cp -Ru "$(CEF_PATH)/locales" "$(RELEASE_DIR)/"; fi

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

package: release _sync-runtime-release ## Stage a runnable folder at dist/rustyteams/.
	@rm -rf $(DIST_DIR)/$(APP_NAME)
	@mkdir -p $(DIST_DIR)/$(APP_NAME)
	@cp -R $(RELEASE_DIR)/. $(DIST_DIR)/$(APP_NAME)/
	@echo ">> packaged to $(DIST_DIR)/$(APP_NAME)/"

dist: package ## Produce dist/rustyteams.zip.
	@cd $(DIST_DIR) && rm -f $(APP_NAME).zip && \
	  (powershell -NoProfile -Command "Compress-Archive -Path '$(APP_NAME)/*' -DestinationPath '$(APP_NAME).zip' -Force" \
	   || zip -r $(APP_NAME).zip $(APP_NAME))
	@echo ">> $(DIST_DIR)/$(APP_NAME).zip"

installer: package ## Build a Windows installer via NSIS (requires makensis on PATH).
	@command -v makensis >/dev/null 2>&1 || { \
	  echo "makensis not found. Install from https://nsis.sourceforge.io/ or 'winget install NSIS.NSIS'"; \
	  exit 1; \
	}
	makensis -V2 installer.nsi
	@echo ">> installer built in $(DIST_DIR)/"

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
	printf "cargo   : "; $(CARGO) --version || FAIL=1; \
	printf "rustc   : "; rustc --version || FAIL=1; \
	printf "cmake   : "; cmake --version 2>/dev/null | head -n1 || { echo "NOT FOUND (install CMake or VS 'C++ CMake tools for Windows')"; FAIL=1; }; \
	printf "ninja   : "; ninja --version 2>/dev/null || { echo "NOT FOUND (run 'winget install Ninja-build.Ninja' or add VS 'C++ CMake tools for Windows')"; FAIL=1; }; \
	printf "link.exe: "; command -v link.exe >/dev/null 2>&1 && echo "on PATH (good — MSVC toolchain active)" || echo "NOT ON PATH (run this shell from a VS Developer prompt, or build via 'cargo' which picks up MSVC)"; \
	printf "CEF_PATH: %s\n" "$(CEF_PATH)"; \
	if [ -f "$(CEF_PATH)/libcef.dll" ]; then \
	  printf "libcef  : found (%s)\n" "$(CEF_PATH)/libcef.dll"; \
	else \
	  printf "libcef  : NOT FOUND at %s\n" "$(CEF_PATH)/libcef.dll"; \
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
	@echo "CEF_PATH=$(CEF_PATH)"
	@echo "PATH=$(PATH)"
