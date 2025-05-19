use crate::calls::models::{Giveaway, GiveawaysStorage};
use crate::consts::USER_GIVEAWAY_KEY;
use crate::errors::AppResult;
use bb8_redis::RedisConnectionManager;
use bb8_redis::bb8::Pool;
use teloxide::Bot;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageReplyMarkupSetters};
use teloxide::prelude::{CallbackQuery, Requester};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, User};
use uuid::Uuid;

pub mod basic_methods;
pub mod giveaway_methods;
pub mod models;
pub mod types;

pub async fn write_participant(
    pool: Pool<RedisConnectionManager>,
    bot: Bot,
    uuid: Uuid,
    user_id: String,
    from: User,
    q: CallbackQuery,
) -> AppResult<()> {
    let mut conn = pool.get().await?;

    let user_id: u64 = user_id.parse().expect("Cannot parse user_id from string");

    let key = format!("{}{}", USER_GIVEAWAY_KEY, user_id);
    let mut storage = GiveawaysStorage::new(key, &mut conn);

    let giveaway = storage.get(uuid).await?;

    if let Some(giveaway) = giveaway {
        log::info!("Giveaway {} found", uuid);
        if giveaway.check_user(from.clone()) {
            log::info!(
                "User {} already take a part in this giveaway {}",
                user_id,
                uuid
            );

            let timestamp = chrono::Utc::now().timestamp();
            let id_for_callback = format!("j:{}:{}:{}", user_id, uuid, timestamp);

            bot.answer_callback_query(q.id)
                .text("Ти вже взяв участь у розіграші!".to_string())
                .show_alert(true)
                .await?;

            update_count_in_button(bot.clone(), id_for_callback, giveaway).await?;
            return Ok(());
        }

        let mut giveaway = giveaway.clone();

        giveaway.add_participant(from.clone());

        storage.insert(uuid, giveaway.clone(), None).await?;

        log::info!(
            "User {} successfully take a part in giveaway {}",
            from.id,
            uuid
        );

        let timestamp = chrono::Utc::now().timestamp();
        let id_for_callback = format!("j:{}:{}:{}", user_id, uuid, timestamp);

        bot.answer_callback_query(q.id)
            .text("Вітаю! Ти успішно взяв участь у розіграші!".to_string())
            .show_alert(true)
            .await?;

        update_count_in_button(bot.clone(), id_for_callback, giveaway).await?;
    }
    Ok(())
}

pub async fn update_count_in_button(
    bot: Bot,
    callback_data: String,
    giveaway: Giveaway,
) -> AppResult<()> {
    let count = giveaway.get_participants().len();
    let text = format!("Взяти участь ({})", count);

    let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        text,
        callback_data,
    )]]);

    let message = giveaway
        .get_message()
        .clone()
        .expect("Cannot get message")
        .clone();

    bot.edit_message_reply_markup(giveaway.group_id, message.id)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
