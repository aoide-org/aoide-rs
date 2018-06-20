REPO=aoide
NAME=aoide
VERSION=$(shell git rev-parse HEAD)
SEMVER_VERSION=$(shell grep version Cargo.toml | awk -F"\"" '{print $$2}' | head -n 1)
CARGO_CACHE=/tmp/cargo-cache


build:
	mkdir -p ${CARGO_CACHE}
	docker pull \
		docker.io/clux/muslrust
	docker run --rm \
		-v ${CARGO_CACHE}:/root/.cargo \
		-v $$PWD:/volume \
		-w /volume \
		-it clux/muslrust \
		cargo build --release
	docker build \
		-t $(REPO)/$(NAME):$(VERSION) \
		.

stop:
	(docker rm -f $(NAME) 2> /dev/null) || true

run: stop
	docker run -p 8080:8080 -v /tmp:/data -d --name=$(NAME) -t $(REPO)/$(NAME):$(VERSION)

tag-latest:
	docker tag $(REPO)/$(NAME):$(VERSION) $(REPO)/$(NAME):latest
	#docker push $(REPO)/$(NAME):latest

tag-semver:
	if curl -sSL https://registry.hub.docker.com/v1/repositories/$(REPO)/$(NAME)/tags | jq -r ".[].name" | grep -q $(SEMVER_VERSION); then \
		echo "Tag $(SEMVER_VERSION) already exists" && exit 1 ;\
	fi
	docker tag $(REPO)/$(NAME):$(VERSION) $(REPO)/$(NAME):$(SEMVER_VERSION)
	#docker push $(REPO)/$(NAME):$(SEMVER_VERSION)
