use crate::errors::AppResult;
use crate::models::State;
use crate::utils::{init_logging, schema};
use bb8_redis::RedisConnectionManager;
use bb8_redis::bb8::Pool;
use dotenv::dotenv;
use std::sync::Arc;
use teloxide::dispatching::dialogue::serializer::Bincode;
use teloxide::dispatching::dialogue::{ErasedStorage, RedisStorage, Storage};
use teloxide::prelude::*;

mod calls;
mod consts;
mod errors;
mod models;
mod utils;

type MyStorage = Arc<ErasedStorage<State>>;

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenv().ok();
    init_logging()?;
    log::info!("Starting giveaway bot...");

    let bot = Bot::new(dotenv::var("TELOXIDE_TOKEN")?);
    let redis_url = dotenv::var("REDIS_URL")?;

    log::info!("Connecting to Redis at {}", redis_url);

    let redis_pool = Pool::builder()
        .max_size(10)
        .build(RedisConnectionManager::new(redis_url.clone())?)
        .await?;

    let state = Arc::new(State::Start);

    let storage: MyStorage = RedisStorage::open(&redis_url, Bincode)
        .await
        .expect("Cannot open redis storage")
        .erase();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            Arc::clone(&state),
            redis_pool.clone(),
            storage
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}
