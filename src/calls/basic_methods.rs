use crate::calls::giveaway_methods::GIVEAWAY_LIST;
use crate::errors::AppResult;
use crate::models::{Command, MenuCommands};
use crate::models::{MyDialogue, State};
use crate::utils::make_keyboard;
use std::str::FromStr;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::requests::Requester;
use teloxide::utils::command::BotCommands;
use uuid::Uuid;

pub async fn help(bot: Bot, msg: Message) -> AppResult<()> {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> AppResult<()> {
    let keyboard = make_keyboard(vec![
        MenuCommands::CreateGiveaway.to_string(),
        MenuCommands::CancelGiveaway.to_string(),
        MenuCommands::GiveawayList.to_string(),
        MenuCommands::AddGroupId.to_string(),
        MenuCommands::EndGiveaway.to_string(),
    ]);

    let message = msg
        .text()
        .expect("Unexpected string")
        .split('_')
        .collect::<Vec<&str>>();
    if message.len() > 1 {
        let mut giveaway_list = GIVEAWAY_LIST.lock().await;
        let user_id = message[0].trim_start_matches("/start ");
        let id = message[1];
        let uuid = Uuid::from_str(id)?;
        
        let giveaway = giveaway_list.get(&UserId(user_id.parse().expect("Invalid UserId")));
        
        if let Some(giveaway) = giveaway {
            let giveaway = giveaway.get(&uuid);
            if let Some(giveaway) = giveaway {
                if giveaway.check_user(msg.from.clone().expect("Cannot get from field")) {
                    bot.send_message(
                        msg.chat.id,
                        "Ти вже взяв участь у розіграші!"
                            .to_string(),
                    ).await?;
                    return Ok(());
                }
            }
        }

        log::info!(
            "User {} clicked on the button",
            msg.from.clone().expect("Cannot get from field").id
        );

        giveaway_list
            .entry(UserId(user_id.parse().expect("Invalid UserId")))
            .and_modify(|giveaway| {
                giveaway.entry(uuid).and_modify(|giveaway| {
                    giveaway.add_participant(msg.from.clone().expect("Cannot get from field"));
                });
            });

        log::info!(
            "User {} successfully take a part in giveaway {}",
            msg.from.clone().expect("Cannot get from field").id,
            id
        );

        bot.send_message(
            msg.chat.id,
            "Вітаю! Ти успішно взяв участь у розіграші!"
                .to_string(),
        ).await?;
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
