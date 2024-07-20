version=0.0.1
image=filipponik/telegraph
platforms=linux/amd64,linux/arm64

# Build docker images
build:
	docker build . -t $(image):$(version) -t $(image):latest

# Build and push multiplatform docker images
build-multiplatform:
	docker buildx build --platform=$(platforms) -t $(image):$(version) -t $(image):latest --push .

fix:
	cargo fmt
	cargo fix --allow-dirty --allow-staged
	cargo clippy

# Run tests
test:
	cargo test