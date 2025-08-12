version=0.8.3
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
	upx --best --lzma target/*-linux-musl/release/rustamanga-mangalib-parser

fix:
	cargo fmt
	cargo fix --allow-dirty --allow-staged
	cargo clippy -- -W clippy::pedantic -W clippy::nursery -W clippy::unwrap_used -W clippy::panic -W clippy::missing_panics_doc -W clippy::panic_in_result_fn -W clippy::cargo_common_metadata

# Run tests
test:
	cargo test
