use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use serde_json::{json, Value};
use thiserror::Error;

pub mod schema;
pub use schema::{Schema, SchemaError};

#[derive(Error, Debug)]
pub enum SysctlError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },
    
    #[error("JSON conversion error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Schema validation error: {0}")]
    Schema(#[from] SchemaError),
    
    #[error("Multiple schema validation errors: {errors}")]
    MultipleValidationErrors { errors: String },
}

/// sysctl.confの設定を格納する構造体
#[derive(Debug, Clone)]
pub struct SysctlConfig {
    settings: HashMap<String, String>,
}

impl SysctlConfig {
    /// 新しいSysctlConfigインスタンスを作成
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }

    /// io::Readからsysctl.confの内容をパースしてsettingsに格納
    pub fn parse<R: Read>(&mut self, reader: R) -> Result<(), SysctlError> {
        let buf_reader = BufReader::new(reader);
        let mut line_number = 0;

        for line_result in buf_reader.lines() {
            line_number += 1;
            let line = line_result?;
            let trimmed = line.trim();

            // 空行やコメント行をスキップ
            if self.should_skip_line(trimmed) {
                continue;
            }

            // 行をパースして設定に追加
            self.parse_line(trimmed, line_number)?;
        }

        Ok(())
    }

    /// スキーマに対して検証を実行
    pub fn validate_with_schema(&self, schema: &Schema) -> Result<(), SysctlError> {
        match schema.validate(&self.settings) {
            Ok(()) => Ok(()),
            Err(errors) => {
                let error_messages: Vec<String> = errors.iter()
                    .map(|e| e.to_string())
                    .collect();
                Err(SysctlError::MultipleValidationErrors {
                    errors: error_messages.join("; "),
                })
            }
        }
    }

    /// 行をスキップすべきかどうかを判定
    fn should_skip_line(&self, line: &str) -> bool {
        line.is_empty() || line.starts_with('#')
    }

    /// 1行をパースして設定に追加
    fn parse_line(&mut self, line: &str, line_number: usize) -> Result<(), SysctlError> {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        
        if parts.len() != 2 {
            return Err(SysctlError::Parse {
                line: line_number,
                message: "invalid format, expected 'key=value'".to_string(),
            });
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        if key.is_empty() {
            return Err(SysctlError::Parse {
                line: line_number,
                message: "empty key not allowed".to_string(),
            });
        }

        self.settings.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// 設定をJSON形式に変換
    pub fn to_json(&self) -> Result<Value, SysctlError> {
        let mut result = json!({});

        for (key, value) in &self.settings {
            self.set_nested_value(&mut result, key, value);
        }

        Ok(result)
    }

    /// ドット記法のキーを階層構造のJSONに設定
    fn set_nested_value(&self, result: &mut Value, key: &str, value: &str) {
        let keys: Vec<&str> = key.split('.').collect();
        let mut current = result;

        // 最後のキー以外は階層を作成
        for k in &keys[..keys.len() - 1] {
            if !current[k].is_object() {
                current[k] = json!({});
            }
            current = &mut current[k];
        }

        // 最後のキーに値を設定
        if let Some(last_key) = keys.last() {
            current[last_key] = json!(value);
        }
    }

    /// 指定されたキーの値を取得
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// 指定されたキーに値を設定
    pub fn set(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }

    /// 全てのキーのベクタを返す
    pub fn keys(&self) -> Vec<&String> {
        self.settings.keys().collect()
    }

    /// 設定のイテレータを返す
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.settings.iter()
    }

    /// 設定数を返す
    pub fn len(&self) -> usize {
        self.settings.len()
    }

    /// 設定が空かどうかを返す
    pub fn is_empty(&self) -> bool {
        self.settings.is_empty()
    }
}

impl Default for SysctlConfig {
    fn default() -> Self {
        Self::new()
    }
}