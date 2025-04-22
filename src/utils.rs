use crate::calls::basic_methods::{cancel, help, invalid_state, start};
use crate::calls::giveaway_methods::{
    add_group_id, cancel_giveaway, create_giveaway, end_giveaway, started_window,
};
use crate::errors::AppResult;
use crate::models::{Command, State};
use colored::*;
use log::{Level, LevelFilter};
use std::collections::HashMap;
use std::str::FromStr;
use std::thread::ThreadId;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{DpHandlerDescription, UpdateFilterExt, dialogue};
use teloxide::dptree;
use teloxide::dptree::{Handler, case};
use teloxide::prelude::{DependencyMap, Update};
use teloxide::types::{KeyboardButton, KeyboardMarkup};

pub fn schema() -> Handler<'static, DependencyMap, AppResult<()>, DpHandlerDescription> {
    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start(start)].endpoint(start)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let callback_query_handler = Update::filter_message()
        .branch(case![State::StartedWindow].endpoint(started_window))
        .branch(case![State::CreateGiveaway].endpoint(create_giveaway))
        .branch(case![State::CancelGiveaway].endpoint(cancel_giveaway))
        .branch(case![State::EndGiveaway].endpoint(end_giveaway))
        .branch(case![State::AddGroupId].endpoint(add_group_id));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(callback_query_handler)
        .branch(dptree::endpoint(invalid_state));

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
}

pub fn make_keyboard(menu_buttons: Vec<String>) -> KeyboardMarkup {
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];

    for menu_button in menu_buttons.chunks(menu_buttons.len()) {
        let row = menu_button
            .iter()
            .map(|version| KeyboardButton::new(version.to_owned()))
            .collect();

        keyboard.push(row);
    }

    KeyboardMarkup::new(keyboard)
}

pub fn init_logging() -> AppResult<()> {
    // Logging lib errors and all app logs
    let log_level = LevelFilter::Debug;

    // This is the main logging dispatch
    let mut main_logging_dispatch = fern::Dispatch::new().level(log_level);

    let stdout_dispatch = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}][{}::{}] {}",
                chrono::Utc::now().format("[%Y-%m-%d][%H:%M:%S%.3f]"),
                parse_thread_id(&std::thread::current().id()),
                match record.level() {
                    Level::Error => format!("{}", record.level()).red(),
                    Level::Warn => format!("{}", record.level()).red().italic(),
                    Level::Info => format!("{}", record.level()).green(),
                    Level::Debug => format!("{}", record.level()).yellow(),
                    Level::Trace => format!("{}", record.level()).bold(),
                },
                record.target(),
                record
                    .line()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "".to_owned()),
                message
            ))
        })
        .chain(std::io::stdout());
    // LevelFilter::from_str()
    main_logging_dispatch = main_logging_dispatch.chain(stdout_dispatch);

    let log_level_for: HashMap<String, String> = HashMap::new();

    for (module, log_level) in log_level_for.into_iter() {
        let log_level = LevelFilter::from_str(&log_level)?;
        main_logging_dispatch = main_logging_dispatch.level_for(module, log_level);
    }

    main_logging_dispatch.apply()?;

    log::info!("Logging level {} enabled", log_level);

    Ok(())
}

fn parse_thread_id(id: &ThreadId) -> String {
    let id_str = format!("{:?}", id);

    let parsed = (|| {
        let start_idx = id_str.find('(')?;
        let end_idx = id_str.rfind(')')?;
        Some(id_str[start_idx + 1..end_idx].to_owned())
    })();

    parsed.unwrap_or(id_str)
}
