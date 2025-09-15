use crate::calls::types::RHashMap;
use redis::aio::MultiplexedConnection;
use redis::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use teloxide::prelude::Message;
use teloxide::types::{InputFile, User, UserId};
use uuid::Uuid;

pub type GiveawaysStorage<'a> = RHashMap<'a, MultiplexedConnection, String, Uuid, Giveaway>;

#[derive(Serialize, Deserialize)]
pub struct GiveawaysList(HashMap<Uuid, Giveaway>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Giveaway {
    pub text: String,
    pub group_id: String,
    pub message: Option<Message>,
    pub photo: String,
    pub owner: User,
    pub participants: Vec<User>,
}

impl Giveaway {
    pub fn new(text: String, photo: String, owner: User) -> Self {
        Giveaway {
            text,
            group_id: String::new(),
            photo,
            message: None,
            owner,
            participants: vec![],
        }
    }

    pub fn add_group_id(&mut self, group_id: String) {
        self.group_id = group_id;
    }

    #[allow(dead_code)]
    pub fn add_participant(&mut self, user: User) {
        self.participants.push(user);
    }

    pub fn get_participants(&self) -> &Vec<User> {
        &self.participants
    }

    pub fn get_owner(&self) -> User {
        self.owner.clone()
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }

    pub fn get_photo(&self) -> InputFile {
        InputFile::file_id(&self.photo)
    }

    pub fn get_message(&self) -> &Option<Message> {
        &self.message
    }

    pub fn set_message(&mut self, message: Message) {
        self.message = Some(message);
    }

    pub fn check_user(&self, user: User) -> bool {
        let user_ids: Vec<UserId> = self.participants.iter().map(|x| x.id).collect();
        user_ids.contains(&user.id)
    }
}

impl ToRedisArgs for GiveawaysList {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let json = serde_json::to_string(self).expect("Failed to serialize Giveaway");
        out.write_arg(json.as_bytes());
    }
}

impl FromRedisValue for GiveawaysList {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let json = String::from_redis_value(v)?;
        serde_json::from_str(&json).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Failed to deserialize Giveaway",
                format!("{:?}", e),
            ))
        })
    }
}

impl ToRedisArgs for Giveaway {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let json = serde_json::to_string(self).expect("Failed to serialize Giveaway");
        out.write_arg(json.as_bytes());
    }
}

impl FromRedisValue for Giveaway {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let json = String::from_redis_value(v)?;
        serde_json::from_str(&json).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Failed to deserialize Giveaway",
                format!("{:?}", e),
            ))
        })
    }
}
