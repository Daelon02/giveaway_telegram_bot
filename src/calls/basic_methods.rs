use crate::calls::write_participant;
use crate::errors::AppResult;
use crate::models::{Command, MenuCommands};
use crate::models::{MyDialogue, State};
use crate::utils::make_keyboard;
use bb8_redis::RedisConnectionManager;
use bb8_redis::bb8::Pool;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::requests::Requester;
use teloxide::utils::command::BotCommands;

pub async fn help(bot: Bot, msg: Message) -> AppResult<()> {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

pub async fn start(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: Pool<RedisConnectionManager>,
) -> AppResult<()> {
    let keyboard = make_keyboard(vec![
        MenuCommands::CreateGiveaway.to_string(),
        MenuCommands::CancelGiveaway.to_string(),
        MenuCommands::GiveawayList.to_string(),
        MenuCommands::AddGroupId.to_string(),
        MenuCommands::EndGiveaway.to_string(),
    ]);

    let from = msg.from.clone().expect("Cannot get from");

    let message = msg
        .text()
        .expect("Unexpected string")
        .split('_')
        .collect::<Vec<&str>>();

    if message.len() > 1 {
        write_participant(pool.clone(), bot.clone(), message, from, msg.chat.id).await?;
        return Ok(());
    } else {
        bot.send_message(
            msg.chat.id,
            "Привіт! Я бот для створення розіграшів! \n\n \
                Тут ти можеш зробити розіграші для свого каналу"
                .to_string(),
        )
        .reply_markup(keyboard.resize_keyboard())
        .await?;

        dialogue.update(State::StartedWindow).await?;
    }

    Ok(())
}

pub async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> AppResult<()> {
    bot.send_message(msg.chat.id, "Cancelling the dialogue.")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

pub async fn invalid_state(bot: Bot, msg: Message) -> AppResult<()> {
    bot.send_message(
        msg.chat.id,
        "Я тебе не розумію, подивись будь-ласка на команду /help",
    )
    .await?;
    Ok(())
}
