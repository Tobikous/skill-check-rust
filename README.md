# sysctl-parser (Rust版)

sysctl.confファイルをパースして、HashMap形式で格納するRustライブラリです。

## プロジェクト構造

```
skill-check-rust/
├── Cargo.toml
├── src/
│   ├── lib.rs          # ライブラリのエントリポイント
│   ├── main.rs         # CLIアプリケーション
│   └── parser/
│       └── mod.rs      # パーサーの実装
└── examples/
    └── sysctl.conf     # サンプルファイル
```

## 機能

- sysctl.confファイルの解析
- 設定値をHashMap形式で格納
- JSON形式での出力（階層構造対応）
- コメント行と空行の自動スキップ
- エラーハンドリング（thiserrorを使用）

## インストール

```bash
# プロジェクトのクローン
git clone https://github.com/Tobikous/skill-check-rust.git
cd skill-check-rust

# ビルド
cargo build --release
```

## 使用方法

### ライブラリとして使用

```rust
use skill_check_rust::SysctlConfig;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("config.conf")?;
    let mut config = SysctlConfig::new();
    
    config.parse(file)?;
    
    // 設定値を取得
    for (key, value) in config.iter() {
        println!("{} = {}", key, value);
    }
    
    // JSON形式で出力
    let json_data = config.to_json()?;
    println!("{}", serde_json::to_string_pretty(&json_data)?);
    
    Ok(())
}
```

### コマンドラインツールとして使用

```bash
# ファイルから読み込み
cargo run -- examples/sysctl.conf

# または、ビルド済みバイナリを使用
./target/release/sysctl-parser examples/sysctl.conf

# 標準入力から読み込み
cat config.conf | cargo run -- -
```

## API

### `SysctlConfig`

```rust
// 新しいインスタンスを作成
let mut config = SysctlConfig::new();

// ファイルをパース
config.parse(reader)?;

// 値を取得
if let Some(value) = config.get("net.ipv4.ip_forward") {
    println!("Value: {}", value);
}

// 値を設定
config.set("custom.key".to_string(), "value".to_string());

// すべてのキーを取得
let keys = config.keys();

// JSON形式に変換
let json = config.to_json()?;
```

## エラーハンドリング

このライブラリは`SysctlError`型を使用してエラーを報告します：

- `Io`: ファイルI/Oエラー
- `Parse`: パースエラー（行番号とメッセージ付き）
- `Json`: JSON変換エラー

## 依存関係

- `serde`: シリアライゼーション
- `serde_json`: JSON処理
- `thiserror`: エラー型の定義
- `clap`: コマンドライン引数のパース

## ライセンス

[ライセンスを指定してください]