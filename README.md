# img2jx

[![CI](https://img.shields.io/github/actions/workflow/status/misosoupTgit/img2jx/ci.yml?branch=main&label=CI&logo=github)](https://github.com/misosoupTgit/img2jx/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/misosoupTgit/img2jx?logo=github)](https://github.com/misosoupTgit/img2jx/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE.md)
[![Rust](https://img.shields.io/badge/edition-2021-orange?logo=rust)](https://www.rust-lang.org/)

Image ↔ JSON converter CLI.

## Install

Prebuilt GPU-enabled binaries are available on [Releases](https://github.com/misosoupTgit/img2jx/releases).

```bash
cargo install --git https://github.com/misosoupTgit/img2jx --features gpu
```

## Usage

```bash
img2jx encode photo.png photo.json
img2jx encode photo.png photo.json --pretty
img2jx decode photo.json output.png
img2jx encode huge.png huge.json --backend gpu --threads 16
```

## License

MIT — see [LICENSE.md](LICENSE.md).
