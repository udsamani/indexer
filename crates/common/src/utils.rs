
pub mod timestamp {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<jiff::Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
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
