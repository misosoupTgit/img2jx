# img2jx

画像と JSON の相互変換 CLI ツール。

## ビルド

```bash
# CPU のみ (デフォルト)
cargo build --release

# GPU 対応
cargo build --release --features gpu
```

## 使い方

```bash
# 画像 → JSON
img2jx encode photo.png photo.json
img2jx encode photo.png photo.json --pretty

# JSON → 画像
img2jx decode photo.json output.png

# GPU バックエンド
img2jx encode huge.png huge.json --backend gpu --features gpu

# スレッド数指定
img2jx encode huge.png huge.json --threads 16
```

## ライセンス

MIT — 依存クレートの NOTICE は `cargo about generate about.hbs > NOTICE` で生成。

```bash
cargo deny check
```
