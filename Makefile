version=0.10.2
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

build-linux-static:
	cargo build --release --target x86_64-unknown-linux-musl
	mv target/x86_64-unknown-linux-musl/release/rustamanga-mangalib-parser ./rustamanga-mangalib-parser_x86_64-unknown-linux-musl
	upx --best --lzma ./rustamanga-mangalib-parser_x86_64-unknown-linux-musl
	./rustamanga-mangalib-parser_x86_64-unknown-linux-musl -V

build-macos-static:
	cargo build --release
	mv target/aarch64-apple-darwin/release/rustamanga-mangalib-parser ./rustamanga-mangalib-parser_aarch64-apple-darwin
	./rustamanga-mangalib-parser_aarch64-apple-darwin -V

fix:
	cargo fmt
	cargo fix --allow-dirty --allow-staged
	cargo clippy --all-targets --all-features

# Run tests
test:
	cargo test
