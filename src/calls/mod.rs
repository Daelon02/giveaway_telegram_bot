use crate::calls::models::GiveawaysStorage;
use crate::errors::AppResult;
use bb8_redis::RedisConnectionManager;
use bb8_redis::bb8::Pool;
use std::str::FromStr;
use teloxide::Bot;
use teloxide::payloads::EditMessageReplyMarkupSetters;
use teloxide::prelude::Requester;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, User};
use url::Url;
use uuid::Uuid;

pub mod basic_methods;
pub mod giveaway_methods;
pub mod models;
pub mod types;

pub async fn write_participant(
    pool: Pool<RedisConnectionManager>,
    bot: Bot,
    message: Vec<&str>,
    from: User,
    chat_id: ChatId,
) -> AppResult<()> {
    let mut conn = pool.get().await?;

    let user_id = message[0].trim_start_matches("/start ");
    let id = message[1];
    let uuid = Uuid::from_str(id)?;

    let user_id: u64 = user_id.parse().expect("Cannot parse user_id from string");

    let mut storage = GiveawaysStorage::new(user_id, &mut conn);

    let giveaway = storage.get(uuid).await?;

    if let Some(giveaway) = giveaway {
        if giveaway.check_user(from.clone()) {
            bot.send_message(chat_id, "Ти вже взяв участь у розіграші!".to_string())
                .await?;
            return Ok(());
        }
        log::info!("User {} clicked on the button", from.id);

        let mut giveaway = giveaway.clone();

        giveaway.add_participant(from.clone());

        storage.insert(uuid, giveaway.clone(), None).await?;

        log::info!(
            "User {} successfully take a part in giveaway {}",
            from.id,
            id
        );

        update_count_in_button(bot.clone(), message, from.clone(), pool.clone(), user_id).await?;

        bot.send_message(
            chat_id,
            "Вітаю! Ти успішно взяв участь у розіграші!".to_string(),
        )
        .await?;
    }
    Ok(())
}

pub async fn update_count_in_button(
    bot: Bot,
    message: Vec<&str>,
    from: User,
    pool: Pool<RedisConnectionManager>,
    owner_user_id: u64,
) -> AppResult<()> {
    let id = message[1];
    let uuid = Uuid::from_str(id)?;

    let mut conn = pool.get().await?;

    let mut giveaway = GiveawaysStorage::new(from.id.0, &mut conn);

    let giveaway = giveaway.get(uuid).await?;

    if let Some(giveaway) = giveaway {
        let count = giveaway.get_participants().len();
        let text = format!("Взяти участь ({})", count);

        let url = dotenv::var("BOT_URL").expect("GIVEAWAY_URL must be set");
        let url = Url::from_str(&format!("{}{}_{}", url, owner_user_id, id))?;

        let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(text, url)]]);

        let message = giveaway
            .get_message()
            .clone()
            .expect("Cannot get message")
            .clone();

        bot.edit_message_reply_markup(giveaway.group_id, message.id)
            .reply_markup(keyboard)
            .await?;
    }
    Ok(())
}
