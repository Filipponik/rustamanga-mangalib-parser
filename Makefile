version=0.3.1
image=filipponik/mangalib
platforms=linux/amd64,linux/arm64

# Build docker images
build:
	docker build . -t $(image):$(version) -t $(image):latest

# Build and push multiplatform docker images
build-multiplatform:
	docker buildx build --platform=$(platforms) -t $(image):$(version) -t $(image):latest --push .

build-static:
	docker run -v ./:/volume --rm -t clux/muslrust:stable cargo build --release
	cd target/*-musl/release
	upx --best --lzma mangalib


fix:
	cargo fmt
	cargo fix --allow-dirty --allow-staged
	cargo clippy

# Run tests
test:
	cargo test