# Mangalib parser

## Installing

1. Install Google Chrome
2. Install parser

Clone and build

```shell
git clone git@github.com:Filipponik/mangalib-parser.git
cd mangalib-parser
cargo build --release
cd target/release
```

Or you can just download latest release

## Usage

### Sending resource

```shell
./mangalib send-resource --url=https://example.com
```

### Start web server

````shell
./mangalib serve --port=12345 --browsers=16
````

After this, your app will be available at `http://localhost:{APP_PORT}`

POST /scrap-manga

```json
{
  "slug": "manga-slug",
  "callback_url": "https://example.com"
}
```

### Start RabbitMQ consumer

```shell
./mangalib consume --url=amqp://guest:guest@localhost:5672 --browsers=16
```