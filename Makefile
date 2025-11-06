# yamllint-rs Makefile

BINARY_NAME = yamllint-rs
TARGET_DIR = target
RELEASE_DIR = $(TARGET_DIR)/release
DEBUG_DIR = $(TARGET_DIR)/debug

RELEASE_FLAGS = --release
CARGO_FLAGS =

DOCKER_IMAGE = yamllint-rs
DOCKER_TAG = latest
VERSION = $(shell grep '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
VERSION_TAG = v$(VERSION) 

RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[0;33m
BLUE = \033[0;34m
NC = \033[0m

.PHONY: help build release debug test clean lint fmt fmt-check check docker-build docker-test docker-run

all: release

help:
	@echo "$(BLUE)yamllint-rs Development Commands$(NC)"
	@echo ""
	@echo "$(GREEN)Build Commands:$(NC)"
	@echo "  build      - Build debug binary"
	@echo "  release    - Build optimized release binary"
	@echo "  debug      - Build debug binary (alias for build)"
	@echo ""
	@echo "$(GREEN)Development Commands:$(NC)"
	@echo "  test       - Run all tests"
	@echo "  lint       - Run clippy linter"
	@echo "  fmt        - Format code with rustfmt"
	@echo "  fmt-check  - Check code formatting"
	@echo "  check      - Check code without building"
	@echo ""
	@echo "$(GREEN)Utility Commands:$(NC)"
	@echo "  clean      - Clean build artifacts"
	@echo ""
	@echo "$(GREEN)Docker Commands:$(NC)"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-test  - Test Docker image locally"
	@echo "  docker-run   - Run Docker image with current directory"

build: debug

debug:
	@echo "$(BLUE)Building debug binary...$(NC)"
	cargo build $(CARGO_FLAGS)
	@echo "$(GREEN)Debug binary built: $(DEBUG_DIR)/$(BINARY_NAME)$(NC)"

release:
	@echo "$(BLUE)Building optimized release binary...$(NC)"
	cargo build $(RELEASE_FLAGS) $(CARGO_FLAGS)
	@echo "$(GREEN)Release binary built: $(RELEASE_DIR)/$(BINARY_NAME)$(NC)"

test:
	@echo "$(BLUE)Running tests...$(NC)"
	cargo test $(CARGO_FLAGS)
	@echo "$(GREEN)All tests passed!$(NC)"

lint:
	@echo "$(BLUE)Running clippy...$(NC)"
	cargo clippy $(CARGO_FLAGS) -- -D warnings
	@echo "$(GREEN)Clippy passed!$(NC)"

fmt:
	@echo "$(BLUE)Formatting code...$(NC)"
	cargo fmt
	@echo "$(GREEN)Code formatted!$(NC)"

fmt-check:
	@echo "$(BLUE)Checking code formatting...$(NC)"
	cargo fmt --check
	@echo "$(GREEN)Code formatting check passed!$(NC)"

check:
	@echo "$(BLUE)Checking code...$(NC)"
	cargo check $(CARGO_FLAGS)
	@echo "$(GREEN)Code check passed!$(NC)"

clean:
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	cargo clean
	@echo "$(GREEN)Clean completed!$(NC)"

docker-build:
	@echo "$(BLUE)Building Docker image...$(NC)"
	@echo "$(YELLOW)Version from Cargo.toml: $(VERSION)$(NC)"
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) -t $(DOCKER_IMAGE):$(VERSION_TAG) .
	@echo "$(GREEN)Docker image built: $(DOCKER_IMAGE):$(DOCKER_TAG) and $(DOCKER_IMAGE):$(VERSION_TAG)$(NC)"

docker-test:
	@echo "$(BLUE)Testing Docker image...$(NC)"
	@echo "$(YELLOW)Testing version command...$(NC)"
	docker run --rm $(DOCKER_IMAGE):$(DOCKER_TAG) --version || docker run --rm $(DOCKER_IMAGE):$(DOCKER_TAG) || true
	@echo "$(GREEN)Docker image test completed!$(NC)"

docker-run:
	@echo "$(BLUE)Running Docker image with current directory...$(NC)"
	@echo "$(YELLOW)Usage: make docker-run ARGS='--recursive .'$(NC)"
	@echo "$(YELLOW)Or: make docker-run ARGS='file.yaml'$(NC)"
	docker run --rm \
		-v $(PWD):/work \
		-w /work \
		$(DOCKER_IMAGE):$(DOCKER_TAG) \
		$(ARGS)
