use std::fmt::Debug;
use std::time::Duration;

use redis::ToRedisArgs;
use redis::{AsyncCommands, ExpireOption};
use serde::{Serialize, de::DeserializeOwned};

use crate::errors::AppResult;

/// # Basic commands
/// https://redis.io/docs/latest/develop/data-types/hashes/
/// - HSET: sets the value of one or more fields on a hash.
/// - HGET: returns the value at a given field.
/// - HMGET: returns the values at one or more given fields.
/// - HINCRBY: increments the value at a given field by the integer provided.
/// - HLEN: returns the number of fields contained in the hash stored at key.
/// - HEXISTS: returns if field is an existing field in the hash stored at key.
/// - HEXPIRE: sets an expiration on the field.
/// - HDEL: removes the specified fields from the hash stored at key.
pub struct RHashMap<'a, C, K, F, V> {
    pub key: K,
    pub con: &'a mut C,
    pub _marker: std::marker::PhantomData<(F, V)>,
}

impl<'a, C, K, F, V> RHashMap<'a, C, K, F, V>
where
    C: AsyncCommands,
    K: ToRedisArgs + Send + Sync + Debug,
    F: DeserializeOwned + Serialize,
    V: Serialize + DeserializeOwned,
{
    pub fn new(key: K, con: &'a mut C) -> Self {
        RHashMap {
            key,
            con,
            _marker: std::marker::PhantomData,
        }
    }

    /// Set a field-value pair
    ///
    /// ### Redis Command
    /// HSET
    pub async fn insert(&mut self, field: F, value: V, ttl: Option<Duration>) -> AppResult<()> {
        let value = serde_json::to_string(&value)?;
        let field = serde_json::to_string(&field)?;
        if let Some(ttl) = ttl {
            redis::pipe()
                .atomic()
                .hset(&self.key, &field, value)
                .hpexpire(&self.key, ttl.as_millis() as i64, ExpireOption::NONE, field)
                .query_async(self.con)
                .await
                .map_err(Into::into)
        } else {
            self.con
                .hset::<_, _, _, ()>(&self.key, field, value)
                .await
                .map_err(Into::into)
        }
    }

    /// Get a value by key
    ///
    /// ### Redis Command
    /// HGET
    pub async fn get(&mut self, field: F) -> AppResult<Option<V>> {
        let field = serde_json::to_string(&field)?;
        let value: Option<String> = self.con.hget(&self.key, field).await?;

        match value {
            Some(value) => {
                let value: V = serde_json::from_str(&value)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Get multiple values by keys
    ///
    /// ### Redis Command
    /// HGETALL
    pub async fn get_all(&mut self) -> AppResult<Vec<(F, V)>> {
        log::info!("[RHashMap] get_all by key {:?}", self.key);
        let values: Option<Vec<(String, String)>> = self.con.hgetall(&self.key).await?;

        let mut result = Vec::new();
        if let Some(values) = values {
            for (field, value) in values {
                let value: V = serde_json::from_str(&value)?;
                let field: F = serde_json::from_str(&field)?;
                result.push((field, value));
            }
            Ok(result)
        } else {
            Ok(vec![])
        }
    }

    /// Remove a key
    ///
    /// ### Redis Command
    /// HDEL
    pub async fn remove(&mut self, field: F) -> AppResult<()> {
        let field = serde_json::to_string(&field)?;
        self.con.hdel(&self.key, field).await.map_err(Into::into)
    }
}
