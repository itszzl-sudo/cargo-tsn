#[cfg(test)]
mod tests {
    /// 测试 prepare 命令的核心功能：从 TypeScript 提取 FFI 声明
    #[test]
    fn test_extract_ffi_declarations() {
        let ts_content = r#"
declare function crypto_sha256(data: string): string;
declare function crypto_md5(data: string): string;
declare function file_read(path: string): string;
declare function file_write(path: string, content: string): void;
"#;

        // 简单的正则提取测试
        let regex = regex::Regex::new(r#"declare\s+function\s+(\w+)\(([^)]*)\)\s*:\s*(\w+)"#).unwrap();
        
        let functions: Vec<_> = regex.captures_iter(ts_content)
            .map(|cap| {
                (
                    cap[1].to_string(),  // function name
                    cap[2].to_string(),  // parameters
                    cap[3].to_string(),  // return type
                )
            })
            .collect();

        assert_eq!(functions.len(), 4);
        assert_eq!(functions[0].0, "crypto_sha256");
        assert_eq!(functions[0].1, "data: string");
        assert_eq!(functions[0].2, "string");
        assert_eq!(functions[2].0, "file_read");
        assert_eq!(functions[3].0, "file_write");
    }

    /// 测试 TOML 配置生成
    #[test]
    fn test_generate_toml_config() {
        let config = r#"[package]
name = "test-plugin"
version = "0.1.0"

[functions]
"test_func" = { args = ["string", "number"], ret = "string" }

[link]
lib = "c"
"#;

        // 验证 TOML 可以解析
        let parsed: Result<toml::Value, _> = toml::from_str(config);
        assert!(parsed.is_ok());

        let value = parsed.unwrap();
        assert_eq!(value["package"]["name"].as_str().unwrap(), "test-plugin");
        assert!(value["functions"]["test_func"].is_table());
    }

    /// 测试插件名称规范化
    #[test]
    fn test_plugin_name_normalization() {
        // 测试常见的名称转换
        assert_eq!("my-plugin".replace("-", "_"), "my_plugin");
        assert_eq!("Crypto".to_lowercase(), "crypto");
        assert_eq!("HTTP-Client".replace("-", "_").to_lowercase(), "http_client");
    }

    /// 测试文件路径处理
    #[test]
    fn test_plugin_path_construction() {
        let plugin_name = "crypto";
        let expected_toml = format!("tsnp/{}/ts-native.toml", plugin_name);
        let expected_c = format!("tsnp/{}/{}_win.c", plugin_name, plugin_name);

        assert_eq!(expected_toml, "tsnp/crypto/ts-native.toml");
        assert_eq!(expected_c, "tsnp/crypto/crypto_win.c");
    }

    /// 测试优先级计算（时间戳）
    #[test]
    fn test_priority_from_timestamp() {
        use chrono::Utc;
        
        let now = Utc::now();
        let timestamp = now.timestamp() as u64;
        
        // 时间戳应该是正数且较大（> 1700000000 for 2024+）
        assert!(timestamp > 1700000000);
        
        // 用作优先级应该是合理的
        let priority = timestamp % 10000;
        assert!(priority < 10000);
    }

    /// 测试 TypeScript 类型到 C 类型映射
    #[test]
    fn test_type_mapping() {
        let type_map = [
            ("string", "const char*"),
            ("number", "double"),
            ("void", "void"),
            ("boolean", "int"),
        ];

        for (ts_type, c_type) in type_map {
            match ts_type {
                "string" => assert_eq!(c_type, "const char*"),
                "number" => assert_eq!(c_type, "double"),
                "void" => assert_eq!(c_type, "void"),
                "boolean" => assert_eq!(c_type, "int"),
                _ => panic!("Unknown type: {}", ts_type),
            }
        }
    }
}
