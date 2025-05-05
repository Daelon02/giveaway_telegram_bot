use std::fmt::Display;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;
use teloxide::utils::command::BotCommands;

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Доступні команди:")]
pub enum Command {
    #[command(description = "Показує доступні команди.")]
    Help,
    #[command(description = "Запускає бота.")]
    Start(String),
    #[command(description = "Нічого не робить.")]
    Cancel,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    CreateGiveaway,
    CancelGiveaway,
    StartedWindow,
    AddGroupId,
    EndGiveaway,
    RerollOrEnd,
    List,
    ShowParticipants,
}

pub enum MenuCommands {
    CreateGiveaway,
    CancelGiveaway,
    GiveawayList,
    AddGroupId,
    EndGiveaway,
    DoNothing,
}

impl Display for MenuCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuCommands::CreateGiveaway => write!(f, "Створити розіграш"),
            MenuCommands::CancelGiveaway => write!(f, "Скасувати розіграш"),
            MenuCommands::GiveawayList => write!(f, "Список розіграшів"),
            MenuCommands::AddGroupId => write!(f, "Додати розіграш в групу"),
            MenuCommands::EndGiveaway => write!(f, "Закінчити розіграш"),
            MenuCommands::DoNothing => write!(f, "Do nothing"),
        }
    }
}

impl From<String> for MenuCommands {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Створити розіграш" => MenuCommands::CreateGiveaway,
            "Скасувати розіграш" => MenuCommands::CancelGiveaway,
            "Список розіграшів" => MenuCommands::GiveawayList,
            "Додати розіграш в групу" => MenuCommands::AddGroupId,
            "Закінчити розіграш" => MenuCommands::EndGiveaway,
            _ => MenuCommands::DoNothing,
        }
    }
}

pub enum RerollCommands {
    Reroll,
    End,
}

impl Display for RerollCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RerollCommands::Reroll => write!(f, "Реролл"),
            RerollCommands::End => write!(f, "Закінчити"),
        }
    }
}

impl From<String> for RerollCommands {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Реролл" => RerollCommands::Reroll,
            "Закінчити" => RerollCommands::End,
            _ => RerollCommands::End,
        }
    }
}

pub enum ListCommands {
    ShowParticipants,
    Return,
}

impl Display for ListCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListCommands::ShowParticipants => write!(f, "Показати учасників"),
            ListCommands::Return => write!(f, "Повернутись назад"),
        }
    }
}

impl From<String> for ListCommands {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Показати учасників" => ListCommands::ShowParticipants,
            "Повернутись назад" => ListCommands::Return,
            _ => ListCommands::Return,
        }
    }
}
