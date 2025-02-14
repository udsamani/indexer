pub mod timestamp_with_tz_serializer {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<jiff::Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }

    pub fn serialize<S>(value: &jiff::Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::Value;

        #[test]
        fn test_deserialize_valid_timestamp() {
            let json = Value::String("2025-02-12T21:12:33.778451Z".to_string());
            let timestamp = deserialize(&json).unwrap();
            assert_eq!(timestamp.to_string(), "2025-02-12T21:12:33.778451Z");
        }

        #[test]
        fn test_deserialize_invalid_format() {
            let json = Value::String("not-a-timestamp".to_string());
            let result = deserialize(&json);
            assert!(result.is_err());
        }

        #[test]
        fn test_deserialize_invalid_date() {
            let json = Value::String("2025-13-12T21:12:33.778451Z".to_string()); // Invalid month
            let result = deserialize(&json);
            assert!(result.is_err());
        }
    }
}


pub mod timestamp_millis_serializer {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<jiff::Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = u64::deserialize(deserializer)?;
        jiff::Timestamp::from_millisecond(s as i64)
            .map_err(serde::de::Error::custom)
    }

    #[allow(unused)]
    pub fn serialize<S>(value: &jiff::Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::{Number, Value};

        #[test]
        fn test_deserialize_valid_timestamp() {
            let json = Value::Number(Number::from(1713123153778_u64));
            let timestamp = deserialize(&json).unwrap();
            assert_eq!(timestamp.to_string(), "2024-04-14T19:32:33.778Z");
        }

        #[test]
        fn test_deserialize_invalid_format() {
            let json = Value::String("not-a-timestamp".to_string());
            let result = deserialize(&json);
            assert!(result.is_err());
        }

        #[test]
        fn test_deserialize_invalid_date() {
            let json = Value::String("2025-13-12T21:12:33.778451Z".to_string()); // Invalid month
            let result = deserialize(&json);
            assert!(result.is_err());
        }
    }
}



pub mod static_config {
    use std::collections::HashMap;
    use std::env::vars;

    use config::{builder::DefaultState, Config, ConfigBuilder, Environment};

    pub type CfgBuilder = ConfigBuilder<DefaultState>;

    /// Create a new configuration builder
    ///
    /// It updates the environment variables from the provided path if it exists
    ///
    /// Variables are read from the environment variables in any case.
    pub fn create_config(env_path: &str) -> CfgBuilder {
        dotenvy::from_path_override(env_path).ok();
        let env_vars = vars().collect::<HashMap<String, String>>();
        let source = Environment::default()
            .source(Some(env_vars));
        Config::builder().add_source(source)
    }
}
