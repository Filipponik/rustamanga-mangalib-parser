version=0.10.0
image=filipponik/rustamanga-mangalib-parser
platforms=linux/amd64,linux/arm64

# Build docker images
build:
	docker build . -t $(image):$(version) -t $(image):latest

# Build and push multiplatform docker images
build-multiplatform:
	docker buildx build --platform=$(platforms) -t $(image):$(version) -t $(image):latest --push .

build-static:
	docker run -v ./:/volume --rm -t clux/muslrust:stable cargo build --release
	upx --best --lzma target/*-linux-musl/release/rustamanga-mangalib-parser

fix:
	cargo fmt
	cargo fix --allow-dirty --allow-staged
	cargo clippy --all-targets --all-features

# Run tests
test:
	cargo test
