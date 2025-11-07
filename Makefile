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

.PHONY: help build release debug test clean lint fmt fmt-check check docker-build docker-test docker-run docker-multi-push build-binaries

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
	@echo "  docker-build      - Build Docker image (single platform, local)"
	@echo "  docker-test        - Test Docker image locally"
	@echo "  docker-run         - Run Docker image with current directory"
	@echo "  docker-multi-push  - Build and push multi-platform image to Docker Hub"
	@echo "  build-binaries     - Build linux/amd64 and linux/arm64 binaries"

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

docker-multi-push:
	@echo "$(BLUE)Building and pushing multi-platform Docker image...$(NC)"
	@docker buildx version > /dev/null 2>&1 || (echo "$(RED)Error: docker buildx not available$(NC)" && exit 1)
	@if [ -z "$(DOCKER_HUB_USER)" ]; then \
		echo "$(RED)Error: DOCKER_HUB_USER not set$(NC)"; \
		echo "$(YELLOW)Example: DOCKER_HUB_USER=avnerner make docker-multi-push$(NC)"; \
		exit 1; \
	fi
	@echo "$(YELLOW)Version from Cargo.toml: $(VERSION)$(NC)"
	@echo "$(YELLOW)Platforms: linux/amd64, linux/arm64$(NC)"
	@echo "$(YELLOW)Docker Hub: $(DOCKER_HUB_USER)/$(DOCKER_IMAGE)$(NC)"
	docker buildx build --platform linux/amd64,linux/arm64 \
		-t $(DOCKER_HUB_USER)/$(DOCKER_IMAGE):$(DOCKER_TAG) \
		-t $(DOCKER_HUB_USER)/$(DOCKER_IMAGE):$(VERSION_TAG) \
		--push .
	@echo "$(GREEN)Multi-platform image pushed: $(DOCKER_HUB_USER)/$(DOCKER_IMAGE):$(DOCKER_TAG) and $(DOCKER_HUB_USER)/$(DOCKER_IMAGE):$(VERSION_TAG)$(NC)"

build-binaries:
	@echo "$(BLUE)Building linux/amd64 binary...$(NC)"
	@docker buildx build --platform linux/amd64 --target builder --load -t $(BINARY_NAME):builder-amd64 .
	@docker create --name temp-amd64 --platform linux/amd64 $(BINARY_NAME):builder-amd64
	@docker cp temp-amd64:/build/target/release/$(BINARY_NAME) ./$(BINARY_NAME)-linux-amd64
	@docker rm temp-amd64
	@chmod +x ./$(BINARY_NAME)-linux-amd64
	@echo "$(GREEN)linux/amd64 binary built: $(BINARY_NAME)-linux-amd64$(NC)"
	@echo "$(BLUE)Building linux/arm64 binary...$(NC)"
	@docker buildx build --platform linux/arm64 --target builder --load -t $(BINARY_NAME):builder-arm64 .
	@docker create --name temp-arm64 --platform linux/arm64 $(BINARY_NAME):builder-arm64
	@docker cp temp-arm64:/build/target/release/$(BINARY_NAME) ./$(BINARY_NAME)-linux-arm64
	@docker rm temp-arm64
	@chmod +x ./$(BINARY_NAME)-linux-arm64
	@echo "$(GREEN)linux/arm64 binary built: $(BINARY_NAME)-linux-arm64$(NC)"
