use clap::Parser;
use skill_check_rust::{SysctlConfig, SysctlError, Schema};
use std::fs::File;
use std::io::{self, stdin};

#[derive(Parser, Debug)]
#[command(name = "sysctl-parser")]
#[command(about = "Parse sysctl.conf files", long_about = None)]
struct Args {
    filename: String,
    
    #[arg(short, long)]
    schema: Option<String>,
}

fn main() {
    let args = Args::parse();
    
    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), SysctlError> {
    let mut config = SysctlConfig::new();

    if args.filename == "-" {
        // 標準入力から読み込み
        config.parse(stdin())?;
    } else {
        // ファイルから読み込み
        let file = File::open(&args.filename)
            .map_err(|e| io::Error::new(io::ErrorKind::NotFound, 
                format!("ファイルを開けませんでした: {}", e)))?;
        config.parse(file)?;
    }

    // スキーマ検証（スキーマファイルが指定されている場合）
    if let Some(schema_path) = args.schema {
        println!("スキーマファイルを読み込み中: {}", schema_path);
        let schema = Schema::from_file(&schema_path)?;
        
        println!("設定値をスキーマに対して検証中...");
        match config.validate_with_schema(&schema) {
            Ok(()) => {
                println!("✅ スキーマ検証に成功しました！");
            }
            Err(e) => {
                eprintln!("❌ スキーマ検証エラー: {}", e);
                std::process::exit(1);
            }
        }
    }

    // 結果を出力
    print_results(&config)?;
    Ok(())
}

fn print_results(config: &SysctlConfig) -> Result<(), SysctlError> {
    // 設定値を表示
    println!("読み込んだ設定数: {}\n", config.len());

    for (key, value) in config.iter() {
        println!("{} = {}", key, value);
    }

    // JSON形式で出力
    let json_result = config.to_json()?;
    let json_string = serde_json::to_string_pretty(&json_result)?;
    
    println!("\nJSON形式:");
    println!("{}", json_string);

    Ok(())
}