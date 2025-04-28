use crate::errors::AppResult;
use crate::models::{MenuCommands, MyDialogue, State};
use lazy_static::lazy_static;
use rand::prelude::SliceRandom;
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use std::str::FromStr;
use teloxide::Bot;
use teloxide::payloads::{SendMessageSetters, SendPhotoSetters};
use teloxide::prelude::{Message, Requester, UserId};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile, ParseMode, User};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

lazy_static! {
    pub static ref GIVEAWAY_LIST: Mutex<HashMap<UserId, HashMap<Uuid, Giveaway>>> =
        Mutex::new(HashMap::new());
}

#[derive(Clone, Debug)]
pub struct Giveaway {
    text: String,
    group_id: String,
    photo: InputFile,
    owner: User,
    participants: Vec<User>,
}

impl Giveaway {
    pub fn new(text: String, photo: InputFile, owner: User) -> Self {
        Giveaway {
            text,
            group_id: String::new(),
            photo,
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

    pub fn get_photo(&self) -> &InputFile {
        &self.photo
    }

    pub fn get_winners(&self, count: usize) -> Vec<User> {
        if self.participants.is_empty() {
            return vec![];
        }
        let mut winners = vec![];
        let mut rng = rand::rng();
        let mut indices: Vec<usize> = (0..self.participants.len()).collect();
        indices.shuffle(&mut rng);
        for i in indices.iter().take(count.min(self.participants.len())) {
            winners.push(self.participants[indices[*i]].clone());
        }
        winners
    }
    
    pub fn check_user(&self, user: User) -> bool {
        self.participants.iter().any(|u| u.id == user.id)
    }
}

pub async fn started_window(bot: Bot, dialogue: MyDialogue, msg: Message) -> AppResult<()> {
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
            let giveaway_list = GIVEAWAY_LIST.lock().await;

            let giveaway_list =
                match giveaway_list.get(&msg.from.expect("Cannot get from field").id) {
                    Some(giveaway_list) => giveaway_list,
                    None => {
                        bot.send_message(msg.chat.id, "Немає активних розіграшів")
                            .await?;
                        return Ok(());
                    }
                };

            if giveaway_list.is_empty() {
                bot.send_message(msg.chat.id, "Немає активних розіграшів")
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Виберіть ID розіграшу, який хочете скасувати")
                    .await?;
                get_giveaway_list(bot, giveaway_list.iter(), msg.chat.id.to_string()).await?;
            }
            dialogue.update(State::CancelGiveaway).await?;
        }
        MenuCommands::GiveawayList => {
            let giveaway_list = GIVEAWAY_LIST.lock().await;
            if let Some(giveaway_list) =
                giveaway_list.get(&msg.from.expect("Cannot get from field").id)
            {
                if giveaway_list.is_empty() {
                    bot.send_message(msg.chat.id, "Немає активних розіграшів")
                        .await?;
                } else {
                    for (id, giveaway) in giveaway_list.iter() {
                        let photo = giveaway.get_photo().clone();

                        let text = get_giveaway_content(id, giveaway);
                        bot.send_photo(msg.chat.id, photo)
                            .caption(text)
                            .parse_mode(ParseMode::Html)
                            .await?;
                    }
                }
            } else {
                bot.send_message(msg.chat.id, "Немає активних розіграшів")
                    .await?;
            };

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
            let giveaway_list = GIVEAWAY_LIST.lock().await;
            let giveaway_list = giveaway_list
                .get(&msg.from.expect("Cannot get from field").id)
                .expect("Cannot get giveaway list");
            if giveaway_list.is_empty() {
                bot.send_message(msg.chat.id, "Немає активних розіграшів")
                    .await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    "Виберіть ID розіграшу, який хочете закінчити\n\
                та скільки переможців повинно бути\n\
                приклад: 1234567890 3",
                )
                .await?;
                get_giveaway_list(bot, giveaway_list.iter(), msg.chat.id.to_string()).await?;
            }
            dialogue.update(State::EndGiveaway).await?;
        }
        _ => {
            dialogue.update(State::StartedWindow).await?;
        }
    }
    Ok(())
}

pub async fn create_giveaway(bot: Bot, dialogue: MyDialogue, msg: Message) -> AppResult<()> {
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
        InputFile::file_id(photos.file.id),
        msg.from.clone().expect("Cannot find from"),
    );

    let mut giveaway_list_with_user = GIVEAWAY_LIST.lock().await;
    let mut giveaway_list = giveaway_list_with_user
        .get(&msg.from.clone().expect("Cannot get from field").id)
        .unwrap_or(&HashMap::new())
        .clone();

    giveaway_list.insert(id, giveaway);

    giveaway_list_with_user.insert(msg.from.expect("Cannot get from field").id, giveaway_list);

    bot.send_message(msg.chat.id, format!("Розіграш створено, ID: {}", id))
        .await?;

    dialogue.update(State::StartedWindow).await?;
    Ok(())
}

pub async fn add_group_id(bot: Bot, dialogue: MyDialogue, msg: Message) -> AppResult<()> {
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

    let mut giveaway_list = GIVEAWAY_LIST.lock().await;
    giveaway_list
        .entry(msg.from.clone().expect("Cannot get from field").id)
        .and_modify(|giveaway| {
            giveaway.entry(id).and_modify(|giveaway| {
                giveaway.add_group_id(channelname.clone());
            });
        });

    let giveaway_list = giveaway_list
        .get(&msg.from.clone().expect("Cannot get from field").id)
        .expect("Cannot get from field");
    let giveaway = giveaway_list.get(&id).expect("Cannot get from field");

    let url = Url::from_str(&format!(
        "https://t.me/ua_giveaway_odn_bot?start={}_{}",
        msg.from.expect("Cannot get from field").id,
        id
    ))?;

    let keyboard =
        InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url("Взяти участь", url)]]);

    let photo = giveaway.get_photo().clone();

    bot.send_photo(channelname.clone(), photo)
        .chat_id(channelname.clone())
        .caption("Натисніть на кнопку, щоб взяти участь у розіграші")
        .reply_markup(keyboard)
        .await?;

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

pub async fn cancel_giveaway(bot: Bot, dialogue: MyDialogue, msg: Message) -> AppResult<()> {
    let mut giveaway_list = GIVEAWAY_LIST.lock().await;
    giveaway_list
        .entry(msg.from.clone().expect("Cannot get from field").id)
        .and_modify(|giveaway| {
            giveaway.remove(&Uuid::from_str(msg.text().unwrap_or_default()).unwrap_or_default());
        });

    bot.send_message(msg.chat.id, "Розіграш було скасовано")
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

async fn get_giveaway_list(
    bot: Bot,
    giveaway_list: Iter<'_, Uuid, Giveaway>,
    chat_id: String,
) -> AppResult<()> {
    for (id, giveaway) in giveaway_list {
        let photo = giveaway.get_photo().clone();
        let text = get_giveaway_content(id, giveaway);
        bot.send_photo(chat_id.clone(), photo)
            .caption(text)
            .parse_mode(ParseMode::Html)
            .await?;
    }
    Ok(())
}

pub async fn end_giveaway(bot: Bot, dialogue: MyDialogue, msg: Message) -> AppResult<()> {
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

    let mut general_giveaway_list = GIVEAWAY_LIST.lock().await;
    let mut giveaway_list = general_giveaway_list
        .get(&msg.from.clone().expect("Cannot get from field").id)
        .expect("Cannot get from field")
        .clone();

    if let Ok(uuid) = Uuid::parse_str(id_uuid) {
        if let Some(giveaway) = giveaway_list.get_mut(&uuid) {
            let winners = giveaway.get_winners(count);
            if !winners.is_empty() {
                if winners.len() == 1 {
                    let owner_id = winners[0].id;
                    let owner_name = winners[0].clone().username.unwrap_or("учасник".to_string());
                    let mention =
                        format!("<a href=\"tg://user?id={}\">{}</a>", owner_id, owner_name);
                    bot.send_message(
                        msg.chat.id,
                        format!("Переможець розіграшу {}: {}", uuid, mention),
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                    bot.send_message(
                        giveaway.group_id.clone(),
                        format!("Переможець розіграшу {}: {}", uuid, mention),
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                } else {
                    for winner in winners {
                        let owner_id = winner.id;
                        let owner_name = winner.username.unwrap_or("".to_string());
                        let mention =
                            format!("<a href=\"tg://user?id={}\">{}</a>", owner_id, owner_name);
                        bot.send_message(
                            msg.chat.id,
                            format!("Переможець розіграшу {}: {}", uuid, mention),
                        )
                        .parse_mode(ParseMode::Html)
                        .await?;
                        bot.send_message(
                            giveaway.group_id.clone(),
                            format!("Переможець розіграшу {}: {}", uuid, mention),
                        )
                        .parse_mode(ParseMode::Html)
                        .await?;
                    }
                };
                general_giveaway_list
                    .entry(msg.from.clone().expect("Cannot get from field").id)
                    .and_modify(|giveaway_list| {
                        giveaway_list.remove(&uuid);
                    });
            } else {
                bot.send_message(msg.chat.id, "Немає учасників").await?;
            }
        } else {
            bot.send_message(msg.chat.id, "Невірний ID розіграшу")
                .await?;
        }
    } else {
        bot.send_message(msg.chat.id, "Невірний формат ID").await?;
    }

    dialogue.update(State::StartedWindow).await?;
    Ok(())
}
