use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, de::DeserializeOwned};
use tauri::Manager;
use crate::APP_HANDLE;

const CONFIG_FILE: &str = "settings.toml";

/// 配置管理器，使用 TOML 格式存储配置
pub struct ConfigHelper {
    path: PathBuf,
    configs: HashMap<String, toml::Value>,
    dirty: bool,
}



impl Default for ConfigHelper {
    fn default() -> Self {
        let base_dir = APP_HANDLE.wait().path().app_config_dir().unwrap();
        let path = base_dir.join(CONFIG_FILE);
        let mut instance = Self {
            path,
            configs: HashMap::new(),
            dirty: false,
        };
        instance.load();
        instance
    }
}

impl Drop for ConfigHelper {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.save();
        }
    }
}

impl ConfigHelper {
    /// 从文件加载配置
    pub fn load(&mut self) {
        if !self.path.exists() {
            return;
        }

        match fs::read_to_string(&self.path) {
            Ok(content) => {
                if let Ok(toml_value) = content.parse::<toml::Value>() {
                    self.parse_toml_value(String::new(), &toml_value);
                }
            }
            Err(e) => {
                eprintln!("Failed to read config file: {}", e);
            }
        }
    }

    /// 递归解析 TOML 值并存储到 configs HashMap
    fn parse_toml_value(&mut self, prefix: String, value: &toml::Value) {
        match value {
            toml::Value::Table(table) => {
                for (key, val) in table {
                    let new_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };

                    match val {
                        toml::Value::Table(_) => {
                            // 嵌套表，继续递归
                            self.parse_toml_value(new_key, val);
                        }
                        _ => {
                            // 叶子节点，直接克隆存储
                            self.configs.insert(new_key, val.clone());
                        }
                    }
                }
            }
            _ => {
                // 顶层非表值
                self.configs.insert(prefix, value.clone());
            }
        }
    }

    /// 保存配置到磁盘
    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.dirty {
            return Ok(());
        }

        // 构建 TOML 结构
        let mut toml_map = toml::value::Table::new();

        for (key, value) in &self.configs {
            // 解析点分隔的命名空间
            let parts: Vec<&str> = key.split('.').collect();
            self.insert_nested(&mut toml_map, &parts, value.clone());
        }

        // 确保目录存在
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // 写入文件
        let content = toml::to_string_pretty(&toml_map)?;
        fs::write(&self.path, content)?;

        self.dirty = false;
        Ok(())
    }

    /// 将值插入嵌套的 TOML 表结构，用于构建Table
    fn insert_nested(&self, table: &mut toml::value::Table, parts: &[&str], value: toml::Value) {
        if parts.len() == 1 {
            table.insert(parts[0].to_string(), value);
        } else if parts.len() > 1 {
            let next = parts[0].to_string();
            let rest = &parts[1..];

            if !table.contains_key(&next) {
                table.insert(next.clone(), toml::Value::Table(toml::value::Table::new()));
            }

            if let Some(toml::Value::Table(nested)) = table.get_mut(&next) {
                self.insert_nested(nested, rest, value);
            }
        }
    }

    /// 获取配置值，如果不存在则返回默认值
    /// 支持获取父节点，返回包含所有子项的表
    pub fn get_value<T>(&self, namespace: &str, default: T) -> T
    where
        T: Serialize + DeserializeOwned,
    {
        // 先尝试直接获取叶子节点
        if let Some(v) = self.configs.get(namespace) {
            if let Ok(parsed) = v.clone().try_into() {
                return parsed;
            }
        }

        // 如果是父节点，收集所有子项
        let prefix = if namespace.is_empty() {
            String::new()
        } else {
            format!("{}.", namespace)
        };

        let mut table = toml::value::Table::new();
        for (key, value) in &self.configs {
            if key.starts_with(&prefix) {
                // 获取相对路径并构建嵌套结构
                let rest = &key[prefix.len()..];
                self.insert_nested(&mut table, &rest.split('.').collect::<Vec<_>>(), value.clone());
            }
        }

        // 如果找到了子项，尝试转换为目标类型
        if !table.is_empty() {
            if let Ok(parsed) = toml::Value::Table(table).try_into() {
                return parsed;
            }
        }

        default
    }

    /// 设置配置值
    pub fn set_value<T>(&mut self, namespace: &str, value: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Serialize,
    {
        let toml_value = toml::Value::try_from(value)?;
        self.configs.insert(namespace.to_string(), toml_value);
        self.dirty = true;
        Ok(())

    }

    /// 直接设置 TOML 值（用于复杂类型）
    pub fn set_raw_value(&mut self, namespace: &str, value: toml::Value) {
        self.configs.insert(namespace.to_string(), value);
        self.dirty = true;
    }

    /// 获取原始 TOML 值
    pub fn get_raw_value(&self, namespace: &str) -> Option<&toml::Value> {
        self.configs.get(namespace)
    }

    /// 获取配置文件路径
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[cfg(test)]
    fn with_path(path: PathBuf) -> Self {
        Self {
            path,
            configs: HashMap::new(),
            dirty: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_get_default_value() {
        let helper = ConfigHelper::with_path(tempdir().unwrap().path().join("test.toml"));
        let value: String = helper.get_value("nonexistent", "default".to_string());
        assert_eq!(value, "default");
    }

    #[test]
    fn test_set_and_get_value() {
        let mut helper = ConfigHelper::with_path(tempdir().unwrap().path().join("test.toml"));
        helper.set_value("test.key", "hello".to_string());
        let value: String = helper.get_value("test.key", "default".to_string());
        assert_eq!(value, "hello");
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.toml");

        {
            let mut helper = ConfigHelper::with_path(path.clone());
            helper.set_value("app.name", "Focus".to_string());
            helper.set_value("app.version", "1.0.0".to_string());
            helper.set_value("theme.dark", true);
            helper.set_value("app.data.user", "Bob".to_string());
            helper.save().unwrap();
        }

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("name"));
        assert!(content.contains("Focus"));
    }

    #[test]
    fn test_custom_type() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct ThemeConfig {
            background: String,
            foreground: String,
            font_size: u32,
        }

        let mut helper = ConfigHelper::with_path(tempdir().unwrap().path().join("test.toml"));

        let theme = ThemeConfig {
            background: "#1a1a1a".to_string(),
            foreground: "#ffffff".to_string(),
            font_size: 14,
        };
        helper.set_value("theme.custom", theme.clone());

        let loaded: ThemeConfig = helper.get_value("theme.custom", ThemeConfig {
            background: "#000000".to_string(),
            foreground: "#000000".to_string(),
            font_size: 12,
        });

        assert_eq!(loaded.background, "#1a1a1a");
        assert_eq!(loaded.foreground, "#ffffff");
        assert_eq!(loaded.font_size, 14);
    }

    #[test]
    fn test_get_parent_namespace() {
        let mut helper = ConfigHelper::with_path(tempdir().unwrap().path().join("test.toml"));
        helper.set_value("test.key", "value".to_string());
        helper.set_value("test.name", "test".to_string());
        helper.set_value("test.count", 42i64);

        let test_obj: toml::Value = helper.get_value("test", toml::Value::Table(toml::value::Table::new()));
        assert!(test_obj.is_table());

        let table = test_obj.as_table().unwrap();
        assert_eq!(table.get("key").and_then(|v| v.as_str()), Some("value"));
        assert_eq!(table.get("name").and_then(|v| v.as_str()), Some("test"));
        assert_eq!(table.get("count").and_then(|v| v.as_integer()), Some(42));
    }

    #[test]
    fn test_get_parent_as_struct() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct TestConfig {
            key: String,
            name: String,
            count: i64,
        }

        let mut helper = ConfigHelper::with_path(tempdir().unwrap().path().join("test.toml"));
        helper.set_value("test.key", "value".to_string());
        helper.set_value("test.name", "test".to_string());
        helper.set_value("test.count", 42i64);

        let test_config: TestConfig = helper.get_value("test", TestConfig {
            key: "default".to_string(),
            name: "default".to_string(),
            count: 0,
        });

        assert_eq!(test_config.key, "value");
        assert_eq!(test_config.name, "test");
        assert_eq!(test_config.count, 42);
    }

    #[test]
    fn test_recursive_nested_struct() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct Position {
            x: i32,
            y: i32,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct WindowConfig {
            title: String,
            position: Position,
            size: Size,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct Size {
            width: u32,
            height: u32,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct AppConfig {
            name: String,
            window: WindowConfig,
        }

        let mut helper = ConfigHelper::with_path(tempdir().unwrap().path().join("test.toml"));

        helper.set_value("app.name", "Focus".to_string());
        helper.set_value("app.window.title", "Main Window".to_string());
        helper.set_value("app.window.position.x", 100i32);
        helper.set_value("app.window.position.y", 200i32);
        helper.set_value("app.window.size.width", 800u32);
        helper.set_value("app.window.size.height", 600u32);

        let config: AppConfig = helper.get_value("app", AppConfig {
            name: "default".to_string(),
            window: WindowConfig {
                title: "default".to_string(),
                position: Position { x: 0, y: 0 },
                size: Size { width: 0, height: 0 },
            },
        });

        assert_eq!(config.name, "Focus");
        assert_eq!(config.window.title, "Main Window");
        assert_eq!(config.window.position.x, 100);
        assert_eq!(config.window.position.y, 200);
        assert_eq!(config.window.size.width, 800);
        assert_eq!(config.window.size.height, 600);

        let window: WindowConfig = helper.get_value("app.window", WindowConfig {
            title: "default".to_string(),
            position: Position { x: 0, y: 0 },
            size: Size { width: 0, height: 0 },
        });

        assert_eq!(window.title, "Main Window");
        assert_eq!(window.position.x, 100);
        assert_eq!(window.size.width, 800);
    }
}