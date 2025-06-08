use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Validation error for key '{key}': {message}")]
    Validation { key: String, message: String },
    
    #[error("Required key '{key}' is missing")]
    MissingKey { key: String },
    
    #[error("Unknown type '{type_name}' in schema")]
    UnknownType { type_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    String,
    Bool,
    Int,
    Float,
    Optional(Box<SchemaType>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    #[serde(rename = "type")]
    pub field_type: SchemaType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub schema: HashMap<String, SchemaField>,
}

impl Schema {
    /// ファイルからスキーマを読み込み
    pub fn from_file(path: &str) -> Result<Self, SchemaError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let schema: Schema = serde_yaml::from_str(&contents)?;
        Ok(schema)
    }

    /// 設定値をスキーマに対して検証
    pub fn validate(&self, config: &HashMap<String, String>) -> Result<(), Vec<SchemaError>> {
        let mut errors = Vec::new();

        // スキーマで定義されたすべてのフィールドをチェック
        for (key, field) in &self.schema {
            match config.get(key) {
                Some(value) => {
                    // 値の型をチェック
                    if let Err(e) = self.validate_value(key, value, &field.field_type) {
                        errors.push(e);
                    }
                }
                None => {
                    // 必須フィールドが存在しない場合
                    if field.required {
                        errors.push(SchemaError::MissingKey { key: key.clone() });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// 個別の値を型に対して検証
    fn validate_value(&self, key: &str, value: &str, expected_type: &SchemaType) -> Result<(), SchemaError> {
        match expected_type {
            SchemaType::String => {
                // 文字列型の場合、すべての値が有効
                Ok(())
            }
            SchemaType::Bool => {
                // ブール型の場合、true/false, 1/0, on/off, yes/no を受け入れる
                let lower_value = value.to_lowercase();
                if matches!(lower_value.as_str(), "true" | "false" | "1" | "0" | "on" | "off" | "yes" | "no") {
                    Ok(())
                } else {
                    Err(SchemaError::Validation {
                        key: key.to_string(),
                        message: format!("expected boolean value, got '{}'", value),
                    })
                }
            }
            SchemaType::Int => {
                // 整数型の場合
                if value.parse::<i64>().is_ok() {
                    Ok(())
                } else {
                    Err(SchemaError::Validation {
                        key: key.to_string(),
                        message: format!("expected integer value, got '{}'", value),
                    })
                }
            }
            SchemaType::Float => {
                // 浮動小数点型の場合
                if value.parse::<f64>().is_ok() {
                    Ok(())
                } else {
                    Err(SchemaError::Validation {
                        key: key.to_string(),
                        message: format!("expected float value, got '{}'", value),
                    })
                }
            }
            SchemaType::Optional(inner_type) => {
                // オプショナル型の場合、内部型で検証
                self.validate_value(key, value, inner_type)
            }
        }
    }
} 