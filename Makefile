.DEFAULT_GOAL := help
TAG ?= $(shell git rev-parse --short HEAD)


.PHONY: help
help: ## Display this help message
	@./help.sh "$(MAKEFILE_LIST)"


.PHONY: lint
lint: ## Run Linting Checks
	@cargo clippy -- -D warnings


.PHONY: fmt
fmt: ## Format the code
	@cargo fmt

.PHONY: test
test: ## Run the tests
	@cargo test

.PHONY: build
build: ## Build the project
	@cargo build


.PHONY: indexer-image
indexer-image:         ## Build the indexer image
	@docker buildx build \
		-f ./docker/indexer/Dockerfile \
		--tag indexer:$(TAG) \
		--load \
		.


.PHONY: services
services:				## start docker services
	@docker compose -f ./docker/docker-compose.yaml up --remove-orphans -d

.PHONY: services-stop
services-stop:				## stop docker services
	@docker compose -f ./docker/docker-compose.yaml stop

.PHONY: services-clean
services-clean:				## remove docker containers
	@docker compose -f ./docker/docker-compose.yaml stop
	@docker compose -f ./docker/docker-compose.yaml rm -f

.PHONY: etcd-init
etcd-init:				## initialize etcd
	./docker/etcd-init/init.sh