# 任意のファイルをロードし、辞書型・Map等に格納するプログラム

sysctl.confファイルをパースして、HashMap形式で格納するRustライブラリです。

AIでコーディングをさせた後、処理の確認とRust言語の構成を学習しています。

## プロジェクト構造

```
skill-check-rust/
├── Cargo.toml
├── src/
│   ├── lib.rs          # ライブラリのエントリポイント
│   ├── main.rs         # CLIアプリケーション
│   └── parser/
│       ├── mod.rs      # パーサーの実装
│       └── schema.rs   # スキーマ検証機能
└── examples/
    ├── sysctl_valid.conf   # 有効なサンプルファイル
    ├── sysctl_invalid.conf # 無効なサンプルファイル
    └── schema.yaml         # スキーマファイルのサンプル
```

## 機能

- sysctl.confファイルの解析
- 設定値をHashMap形式で格納
- JSON形式での出力（階層構造対応）
- **スキーマファイルによる入力値検証**（NEW！）
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

### スキーマ検証機能（課題1の追加機能）

設定ファイルの内容をスキーマファイルに対して検証できます。

#### スキーマファイルの形式

YAML形式でスキーマを定義します：

```yaml
schema:
  endpoint:
    type: string
    required: true
    description: "サーバーのエンドポイントURL"
  
  debug:
    type: bool
    required: false
    description: "デバッグモードの有効/無効"
  
  log.file:
    type: string
    required: true
    description: "ログファイルのパス"
  
  max.connections:
    type: int
    required: false
    description: "最大接続数"
  
  timeout:
    type: float
    required: false
    description: "タイムアウト（秒）"
```

#### サポートされる型

- `string`: 文字列値
- `bool`: ブール値（true/false, 1/0, on/off, yes/no）
- `int`: 整数値
- `float`: 浮動小数点値

#### コマンドライン使用例

```bash
# スキーマ検証付きでファイルを解析
./target/debug/sysctl-parser examples/sysctl_valid.conf --schema examples/schema.yaml

# または短縮形
./target/debug/sysctl-parser examples/sysctl_valid.conf -s examples/schema.yaml

# スキーマ検証なしの従来の使い方も可能
./target/debug/sysctl-parser examples/sysctl_valid.conf
```

#### 検証結果の例

**成功時:**
```
スキーマファイルを読み込み中: examples/schema.yaml
設定値をスキーマに対して検証中...
✅ スキーマ検証に成功しました！
```

**失敗時:**
```
❌ スキーマ検証エラー: Multiple schema validation errors: 
Validation error for key 'debug': expected boolean value, got 'invalid_value'; 
Required key 'endpoint' is missing
```

### ライブラリとして使用

```rust
use skill_check_rust::{SysctlConfig, Schema};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("config.conf")?;
    let mut config = SysctlConfig::new();
    
    config.parse(file)?;
    
    // スキーマ検証（オプション）
    let schema = Schema::from_file("schema.yaml")?;
    config.validate_with_schema(&schema)?;
    
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
cargo run -- examples/sysctl_valid.conf

# スキーマ検証付きで読み込み
cargo run -- examples/sysctl_valid.conf --schema examples/schema.yaml

# または、ビルド済みバイナリを使用
./target/release/sysctl-parser examples/sysctl_valid.conf -s examples/schema.yaml

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

// スキーマ検証
config.validate_with_schema(&schema)?;

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

### `Schema`

```rust
// スキーマファイルから読み込み
let schema = Schema::from_file("schema.yaml")?;

// 設定を検証
let config: HashMap<String, String> = // ...
schema.validate(&config)?;
```

## エラーハンドリング

このライブラリは以下のエラー型を使用してエラーを報告します：

### `SysctlError`
- `Io`: ファイルI/Oエラー
- `Parse`: パースエラー（行番号とメッセージ付き）
- `Json`: JSON変換エラー
- `Schema`: スキーマ関連エラー
- `MultipleValidationErrors`: 複数の検証エラー

### `SchemaError`
- `Io`: ファイルI/Oエラー
- `Yaml`: YAML パースエラー
- `Validation`: 値の型検証エラー
- `MissingKey`: 必須キーの欠如
- `UnknownType`: 未知の型指定

## 依存関係

- `serde`: シリアライゼーション
- `serde_json`: JSON処理
- `serde_yaml`: YAML処理（スキーマファイル用）
- `thiserror`: エラー型の定義
- `clap`: コマンドライン引数のパース
