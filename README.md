# Mangalib parser

## Installing

```shell
git clone git@github.com:Filipponik/mangalib-parser.git
cd mangalib-parser
cp .env.example .env
cargo build --release
```

## Usage

```shell
cargo run --release
```
After this, your app will be available at http://localhost:{APP_PORT}

POST /scrap-manga
```json
{
  "slug": "manga-slug",
  "callback_url": "https://example.com"
}

```