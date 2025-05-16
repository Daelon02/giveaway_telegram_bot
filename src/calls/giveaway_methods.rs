use crate::calls::models::{Giveaway, GiveawaysStorage};
use crate::calls::write_participant;
use crate::errors::{AppErrors, AppResult};
use crate::models::{ListCommands, MenuCommands, MyDialogue, RerollCommands, State};
use crate::utils::{format_user_mention, make_keyboard};
use bb8_redis::RedisConnectionManager;
use bb8_redis::bb8::Pool;
use std::str::FromStr;
use teloxide::Bot;
use teloxide::payloads::{AnswerCallbackQuerySetters, SendMessageSetters, SendPhotoSetters};
use teloxide::prelude::{CallbackQuery, ChatId, Message, Requester};
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId, ParseMode, User};
use uuid::Uuid;

pub async fn started_window(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: Pool<RedisConnectionManager>,
) -> AppResult<()> {
    let menu = MenuCommands::from(msg.text().unwrap_or_default().to_string());

    match menu {
        MenuCommands::CreateGiveaway => {
            bot.send_message(
                msg.chat.id,
                "Відправ картинки з дописом щоб створити розіграш",
            )
            .await?;
            dialogue.update(State::CreateGiveaway).await?;
        }
        MenuCommands::CancelGiveaway => {
            bot.send_message(msg.chat.id, "Виберіть ID розіграшу, який хочете скасувати")
                .await?;
            get_all_giveaways(bot, msg, pool).await?;
            dialogue.update(State::CancelGiveaway).await?;
        }
        MenuCommands::GiveawayList => {
            let is_not_empty = get_all_giveaways(bot.clone(), msg.clone(), pool).await?;

            if is_not_empty {
                let keyboard = make_keyboard(vec![
                    ListCommands::ShowParticipants.to_string(),
                    ListCommands::Return.to_string(),
                ]);

                bot.send_message(
                    msg.chat.id,
                    "Якщо потрібен повний список учасників, натисни кнопку нижче",
                )
                .reply_markup(keyboard.resize_keyboard())
                .await?;

                dialogue.update(State::List).await?;
                return Ok(());
            }

            dialogue.update(State::StartedWindow).await?;
        }
        MenuCommands::AddGroupId => {
            bot.send_message(
                msg.chat.id,
                "Назву каналу та ID розіграшу через пробіл\n\
                Наприклад: @channelname 1234567890",
            )
            .await?;
            dialogue.update(State::AddGroupId).await?;
        }
        MenuCommands::EndGiveaway => {
            bot.send_message(
                msg.chat.id,
                "Виберіть ID розіграшу, який хочете закінчити\n\
                та скільки переможців повинно бути\n\
                приклад: 1234567890 3",
            )
            .await?;
            get_all_giveaways(bot.clone(), msg.clone(), pool.clone()).await?;
            dialogue.update(State::EndGiveaway).await?;
        }
        _ => {
            dialogue.update(State::StartedWindow).await?;
        }
    }
    Ok(())
}

pub async fn create_giveaway(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: Pool<RedisConnectionManager>,
) -> AppResult<()> {
    log::info!("Creating giveaway by user {:?}", msg.from);

    let photos = match msg.photo() {
        Some(photos) => photos[0].clone(),
        None => {
            bot.send_message(msg.chat.id, "Треба надіслати фото")
                .await?;
            dialogue.update(State::CreateGiveaway).await?;
            return Ok(());
        }
    };

    let text = match msg.caption() {
        Some(text) => text,
        None => {
            bot.send_message(msg.chat.id, "Треба надіслати текст разом з картинками")
                .await?;
            dialogue.update(State::CreateGiveaway).await?;
            return Ok(());
        }
    };

    let id = Uuid::new_v4();

    let giveaway = Giveaway::new(
        text.to_string(),
        photos.file.id,
        msg.from.clone().expect("Cannot find from"),
    );

    let mut conn = pool.get().await?;

    let user_id = msg.from.clone().expect("Cannot get from field").id.0;

    let mut giveaway_list = GiveawaysStorage::new(user_id, &mut conn);

    giveaway_list.insert(id, giveaway, None).await?;

    bot.send_message(msg.chat.id, format!("Розіграш створено, ID: {}", id))
        .await?;

    dialogue.update(State::StartedWindow).await?;
    Ok(())
}

pub async fn add_group_id(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: Pool<RedisConnectionManager>,
) -> AppResult<()> {
    log::info!("Adding group ID to giveaway...");

    let id = match msg.text() {
        Some(id) => id,
        None => {
            bot.send_message(msg.chat.id, "Треба надіслати ID розіграшу")
                .await?;
            dialogue.update(State::StartedWindow).await?;
            return Ok(());
        }
    };

    let id = id.split_whitespace().collect::<Vec<&str>>();

    if id.len() <= 1 {
        bot.send_message(msg.chat.id, "Треба надіслати ID розіграшу")
            .await?;
        dialogue.update(State::StartedWindow).await?;
        return Ok(());
    };

    let channelname = id[0].to_string();

    let id = Uuid::from_str(id[1])?;

    let mut conn = pool.get().await?;

    let from = msg.from.clone().expect("Cannot get from field").id.0;

    let mut storage = GiveawaysStorage::new(from, &mut conn);

    let giveaway = storage.get(id).await?;

    if giveaway.is_none() {
        bot.send_message(msg.chat.id, "Не вдалось знайти розіграш з таким ID")
            .await?;
        dialogue.update(State::StartedWindow).await?;
        return Ok(());
    }

    let mut giveaway = giveaway.expect("Cannot get giveaway from field");

    let photo = giveaway.get_photo();

    giveaway.add_group_id(channelname.clone());

    let url = format!("j:{}:{}", msg.from.expect("Cannot get from field").id, id);

    let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Взяти участь",
        url,
    )]]);

    let m = bot
        .send_photo(channelname.clone(), photo)
        .chat_id(channelname.clone())
        .caption(giveaway.text.clone())
        .reply_markup(keyboard)
        .await?;

    giveaway.set_message(m);

    storage.insert(id, giveaway.clone(), None).await?;

    bot.send_message(
        msg.chat.id,
        format!(
            "Розіграш створено в каналі {} з ID {}",
            giveaway.group_id, id
        ),
    )
    .await?;

    dialogue.update(State::StartedWindow).await?;

    Ok(())
}

pub async fn cancel_giveaway(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: Pool<RedisConnectionManager>,
) -> AppResult<()> {
    let mut conn = pool.get().await?;

    let mut storage = GiveawaysStorage::new(
        msg.from.clone().expect("Cannot get from field").id.0,
        &mut conn,
    );

    storage
        .remove(Uuid::from_str(msg.text().unwrap_or_default()).unwrap_or_default())
        .await?;

    bot.send_message(msg.chat.id, "Розіграш було закінчено")
        .await?;

    dialogue.update(State::StartedWindow).await?;
    Ok(())
}

fn get_giveaway_content(id: &Uuid, giveaway: &Giveaway) -> String {
    let owner_id = giveaway.get_owner().id;
    let owner_name = giveaway
        .get_owner()
        .username
        .unwrap_or("учасник".to_string());
    let mention = format!("<a href=\"tg://user?id={}\">{}</a>", owner_id, owner_name);

    if giveaway.group_id.is_empty() {
        format!(
            "ID: {}\nВласник: {}\nТекст: {}\nУчасники: {}",
            id,
            mention,
            giveaway.get_text(),
            giveaway.get_participants().len(),
        )
    } else {
        format!(
            "ID: {}\nВласник: {}\nТекст: {}\nУчасники: {}\nГрупа: {}",
            id,
            mention,
            giveaway.get_text(),
            giveaway.get_participants().len(),
            giveaway.group_id,
        )
    }
}

pub async fn end_giveaway(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: Pool<RedisConnectionManager>,
) -> AppResult<()> {
    log::info!("Ending giveaway...");

    let id = msg.text().unwrap_or_default();

    let id = id.split_whitespace().collect::<Vec<&str>>();

    if id.len() <= 1 {
        bot.send_message(msg.chat.id, "Треба надіслати ID розіграшу")
            .await?;
        dialogue.update(State::StartedWindow).await?;
        return Ok(());
    };

    let id_uuid = id[0];
    let count = id[1].parse::<usize>().unwrap_or(1);

    let mut conn = pool.get().await?;

    let mut storage = GiveawaysStorage::new(
        msg.from.clone().expect("Cannot get from field").id.0,
        &mut conn,
    );

    let uuid = Uuid::from_str(id_uuid).expect("Cannot get giveaway ID");

    let giveaway = storage.get(uuid).await?;

    if let Some(giveaway) = giveaway {
        let winners = giveaway.get_winners(count);
        get_winner_or_winners(
            winners,
            uuid,
            bot.clone(),
            msg.chat.id,
            giveaway.group_id,
            dialogue,
            giveaway.message.expect("Cannot get message").id,
        )
        .await?;
        let keyboard = make_keyboard(vec![
            RerollCommands::End.to_string(),
            RerollCommands::Reroll.to_string(),
        ]);

        bot.send_message(msg.chat.id, "Якщо ти хочеш остаточно завершити або зробити перевибір переможця/переможців розіграш\n нажми відповідну кнопку")
            .reply_markup(keyboard.resize_keyboard()).await?;
        return Ok(());
    } else {
        bot.send_message(msg.chat.id, "Невірний ID розіграшу")
            .await?;
    }

    dialogue.update(State::StartedWindow).await?;
    Ok(())
}

pub async fn get_all_giveaways(
    bot: Bot,
    msg: Message,
    pool: Pool<RedisConnectionManager>,
) -> AppResult<bool> {
    let mut conn = pool.get().await?;
    let mut storage = GiveawaysStorage::new(
        msg.from.clone().expect("Cannot get from field").id.0,
        &mut conn,
    );

    let giveaways = storage.get_all().await?;

    if giveaways.is_empty() {
        bot.send_message(msg.chat.id, "Немає активних розіграшів")
            .await?;
        return Ok(false);
    } else {
        for (id, giveaway) in giveaways {
            let photo = giveaway.get_photo().clone();
            let text = get_giveaway_content(&id, &giveaway);
            bot.send_photo(msg.chat.id, photo)
                .caption(text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(true)
}

pub async fn get_winner_or_winners(
    winners: Vec<User>,
    uuid: Uuid,
    bot: Bot,
    chat_id: ChatId,
    group_id: String,
    dialogue: MyDialogue,
    reply_to: MessageId,
) -> AppResult<()> {
    if !winners.is_empty() {
        if winners.len() == 1 {
            let mention = format_user_mention(&winners[0]);
            bot.send_message(
                chat_id,
                format!("Переможець розіграшу {}: {}", uuid, mention),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            bot.send_message(group_id, format!("Переможець розіграшу: {}", mention))
                .reply_to(reply_to)
                .parse_mode(ParseMode::Html)
                .await?;
        } else {
            let mut message_with_winners = String::from("Переможці розіграшу:\n");

            let mut index = 1;

            for winner in winners {
                let mention = format_user_mention(&winner);
                message_with_winners.push_str(&format!("{}. {}\n", index, mention));
                index += 1;
            }
            bot.send_message(group_id.clone(), message_with_winners)
                .reply_to(reply_to)
                .parse_mode(ParseMode::Html)
                .await?;
        };
        dialogue.update(State::RerollOrEnd).await?;
    } else {
        bot.send_message(chat_id, "Немає учасників").await?;
    }
    Ok(())
}

pub async fn reroll_or_end(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: Pool<RedisConnectionManager>,
) -> AppResult<()> {
    log::info!("Rerolling or ending giveaway...");

    let keyboard = make_keyboard(vec![
        MenuCommands::CreateGiveaway.to_string(),
        MenuCommands::CancelGiveaway.to_string(),
        MenuCommands::GiveawayList.to_string(),
        MenuCommands::AddGroupId.to_string(),
        MenuCommands::EndGiveaway.to_string(),
    ]);

    let menu = RerollCommands::from(msg.text().unwrap_or_default().to_string());

    match menu {
        RerollCommands::End => {
            bot.send_message(msg.chat.id, "Відправ ID розіграшу, який хочете закінчити")
                .reply_markup(keyboard.resize_keyboard())
                .await?;
            dialogue.update(State::CancelGiveaway).await?;
            Ok(())
        }
        RerollCommands::Reroll => {
            bot.send_message(
                msg.chat.id,
                "Виберіть ID розіграшу, який хочете перевибрати\n\
                та скільки переможців повинно бути\n\
                приклад: 1234567890 3",
            )
            .await?;
            get_all_giveaways(bot.clone(), msg.clone(), pool.clone()).await?;
            dialogue.update(State::EndGiveaway).await?;
            Ok(())
        }
    }
}

pub async fn list(bot: Bot, dialogue: MyDialogue, msg: Message) -> AppResult<()> {
    let menu = ListCommands::from(msg.text().unwrap_or_default().to_string());

    match menu {
        ListCommands::ShowParticipants => {
            bot.send_message(
                msg.chat.id,
                "Виберіть ID розіграшу, учасників якого хочете побачити",
            )
            .await?;
            dialogue.update(State::ShowParticipants).await?;
        }
        ListCommands::Return => {
            let keyboard = make_keyboard(vec![
                MenuCommands::CreateGiveaway.to_string(),
                MenuCommands::CancelGiveaway.to_string(),
                MenuCommands::GiveawayList.to_string(),
                MenuCommands::AddGroupId.to_string(),
                MenuCommands::EndGiveaway.to_string(),
            ]);

            bot.send_message(msg.chat.id, "Повернення назад")
                .reply_markup(keyboard.resize_keyboard())
                .await?;

            dialogue.update(State::StartedWindow).await?;
        }
    }
    Ok(())
}

pub async fn show_participants(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: Pool<RedisConnectionManager>,
) -> AppResult<()> {
    log::info!("Showing participants...");

    let keyboard = make_keyboard(vec![
        MenuCommands::CreateGiveaway.to_string(),
        MenuCommands::CancelGiveaway.to_string(),
        MenuCommands::GiveawayList.to_string(),
        MenuCommands::AddGroupId.to_string(),
        MenuCommands::EndGiveaway.to_string(),
    ]);

    let mut conn = pool.get().await?;

    let mut storage = GiveawaysStorage::new(
        msg.from.clone().expect("Cannot get from field").id.0,
        &mut conn,
    );

    let id = msg.text().unwrap_or_default();

    let id = match Uuid::from_str(id) {
        Ok(id) => id,
        Err(_) => {
            bot.send_message(msg.chat.id, "Невірний ID розіграшу")
                .await?;

            return Ok(());
        }
    };

    let giveaway = storage.get(id).await?;

    if giveaway.is_none() {
        bot.send_message(msg.chat.id, "Невірний ID розіграшу")
            .await?;
        return Ok(());
    }

    let giveaway = giveaway.expect("Cannot get giveaway from field");

    let participants = giveaway.get_participants();

    if participants.is_empty() {
        bot.send_message(msg.chat.id, "Немає учасників")
            .reply_markup(keyboard.resize_keyboard())
            .await?;
        dialogue.update(State::StartedWindow).await?;
        return Ok(());
    }

    let mut message_with_participants = String::from("Учасники розіграшу:\n");

    for (index, participant) in participants.iter().enumerate() {
        let owner_id = participant.id;
        let owner_name = participant
            .username
            .clone()
            .unwrap_or_else(|| participant.first_name.clone());
        let mention = format!("<a href=\"tg://user?id={}\">{}</a>", owner_id, owner_name);
        message_with_participants.push_str(&format!("{}. {}\n", index + 1, mention));
    }

    bot.send_message(msg.chat.id, message_with_participants)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard.resize_keyboard())
        .await?;

    dialogue.update(State::StartedWindow).await?;
    Ok(())
}

pub async fn handle_callback_from_button(
    bot: Bot,
    q: CallbackQuery,
    pool: Pool<RedisConnectionManager>,
) -> AppResult<()> {
    log::info!("Handling callback from button by callback query...");
    if let Some(data) = &q.data {
        if data.starts_with("j:") {
            let parser_string = data.replace("j:", "");
            let user = q.from.clone();

            log::info!("User data: {:?}", parser_string);

            let mut parts = parser_string.splitn(3, ':');

            let user_id_str = parts
                .next()
                .ok_or(AppErrors::StringError("Missing user_id".to_string()))?;

            let uuid_str = parts
                .next()
                .ok_or(AppErrors::StringError("Missing uuid".to_string()))?;

            log::info!("User {} clicked on the button", user.id);

            write_participant(
                pool.clone(),
                bot.clone(),
                Uuid::from_str(uuid_str)?,
                user_id_str.to_string(),
                user,
                q,
            )
            .await?;
        }
    } else {
        bot.answer_callback_query(q.id)
            .show_alert(true)
            .text("Не вдалось знайти розіграш")
            .await?;
    }

    Ok(())
}
