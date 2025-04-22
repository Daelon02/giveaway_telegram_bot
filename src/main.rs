use crate::errors::AppResult;
use crate::models::State;
use crate::utils::{init_logging, schema};
use dotenv::dotenv;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;

mod calls;
mod errors;
mod models;
mod utils;

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenv().ok();
    init_logging()?;
    log::info!("Starting fitness bot...");

    let bot = Bot::new(dotenv::var("TELOXIDE_TOKEN")?);

    let state = Arc::new(State::Start);

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            InMemStorage::<State>::new(),
            Arc::clone(&state)
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}
