use axum::http::Method;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

pub fn serialize_method<S>(method: &Method, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(method.as_str())
}

pub fn deserialize_method<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(D::Error::custom)
}

pub fn serialize_method_map<S, T>(
    map: &HashMap<Method, T>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    use serde::ser::SerializeMap;
    let mut map_ser = serializer.serialize_map(Some(map.len()))?;
    for (k, v) in map {
        map_ser.serialize_entry(k.as_str(), v)?;
    }
    map_ser.end()
}

// 自定义HashMap<Method, T>反序列化
pub fn deserialize_method_map<'de, D, T>(deserializer: D) -> Result<HashMap<Method, T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    use serde::de::Error;
    let map: HashMap<String, T> = HashMap::deserialize(deserializer)?;
    let mut result = HashMap::new();
    for (k, v) in map {
        let method = k.parse().map_err(D::Error::custom)?;
        result.insert(method, v);
    }
    Ok(result)
}
