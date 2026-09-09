use serde::{Serialize, de::DeserializeOwned};

pub trait KeyValueStore {
    fn put_bytes(&mut self, key: &str, value: &[u8]) -> Result<(), String>;
    fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, String>;
    fn delete(&mut self, key: &str) -> Result<(), String>;
}

pub fn put_json<T: Serialize>(
    store: &mut dyn KeyValueStore,
    key: &str,
    value: &T,
) -> Result<(), String> {
    let data = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    store.put_bytes(key, &data)
}

pub fn get_json<T: DeserializeOwned>(
    store: &dyn KeyValueStore,
    key: &str,
) -> Result<Option<T>, String> {
    let Some(data) = store.get_bytes(key)? else {
        return Ok(None);
    };
    serde_json::from_slice(&data)
        .map(Some)
        .map_err(|e| e.to_string())
}
