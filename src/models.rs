use std::fmt::Display;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;
use teloxide::utils::command::BotCommands;

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start the purchase procedure.")]
    Start,
    #[command(description = "cancel the purchase procedure.")]
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
