# img2jx

画像と JSON の相互変換 CLI ツール。

## 使い方

```bash
# 画像 → JSON
img2jx encode photo.png photo.json
img2jx encode photo.png photo.json --pretty

# JSON → 画像
img2jx decode photo.json output.png

# GPU バックエンド
img2jx encode huge.png huge.json --backend gpu

# スレッド数指定
img2jx encode huge.png huge.json --threads 16
```

## ライセンス

MIT — 詳細は [LICENSE.md](LICENSE.md) を参照してください。
