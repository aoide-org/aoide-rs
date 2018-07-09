REPO=aoide.org
NAME=aoide-rs

BUILD_CARGO_CACHE=/tmp/cargo-cache

RUN_DATA_DIR=/tmp
RUN_HTTP_PORT=8080

GIT_VERSION=$(shell git rev-parse HEAD)
CARGO_VERSION=$(shell grep version Cargo.toml | awk -F"\"" '{print $$2}' | head -n 1)

build:
	mkdir -p "$(BUILD_CARGO_CACHE)"
	docker pull docker.io/alpine
	docker pull docker.io/clux/muslrust:stable
	docker run --rm \
		-v "$(BUILD_CARGO_CACHE)":/root/.cargo:Z \
		-v "$$PWD":/volume:Z \
		-w /volume \
		-it clux/muslrust:stable \
		cargo build --release
	docker build \
		-t $(REPO)/$(NAME):$(GIT_VERSION) \
		.

stop:
	(docker rm -f $(NAME) 2> /dev/null) || true

run: stop
	docker run -d -p $(RUN_HTTP_PORT):8080 -v "$(RUN_DATA_DIR)":/aoide/data:Z --user aoide --name=$(NAME) -t $(REPO)/$(NAME):$(GIT_VERSION)

tag-latest:
	docker tag $(REPO)/$(NAME):$(GIT_VERSION) $(REPO)/$(NAME):latest
	#docker push $(REPO)/$(NAME):latest

tag-semver:
	if curl -sSL https://registry.hub.docker.com/v1/repositories/$(REPO)/$(NAME)/tags | jq -r ".[].name" | grep -q $(CARGO_VERSION); then \
		echo "Tag $(CARGO_VERSION) already exists" && exit 1 ;\
	fi
	docker tag $(REPO)/$(NAME):$(GIT_VERSION) $(REPO)/$(NAME):$(CARGO_VERSION)
	#docker push $(REPO)/$(NAME):$(CARGO_VERSION)
