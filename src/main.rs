#![allow(unused_imports)]
use teloxide::{prelude::*, types::{ChatPermissions, Me}, utils::command::BotCommand};
use std::env;
use std::error::Error;
use std::str;
use std::process::Command;
//use self::chrono::prelude::*;
//use self::chrono::{DateTime, Duration, NaiveDateTime, Utc};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Commands {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a macro.", parse_with = "split")]
    Macro {option: String, macro_str: String},
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
    #[command(description = "handle a command admin.")]
    Ban(String),
    #[command(description = "kick a user.")]
    Kick,
}


type Cx = UpdateWithCx<AutoSend<Bot>, Message>;

/*
// Muta un utente rispondendo a un suo messaggio
async fn mute_user(cx: &Cx, time: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cx.update.reply_to_message() {
        Some(msg1) => {
            cx.requester
                .restrict_chat_member(
                    cx.update.chat_id(),
                    msg1.from().expect("Must be MessageKind::Common").id,
                    ChatPermissions::default(),
                )
                .until_date(
                    DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp(cx.update.date as i64, 0),
                        Utc,
                    ) + time,
                )
                .await?;
        }
        None => {
            cx.reply_to("Devi usare questo comando in risposta a un messaggio").send().await?;
        }
    }
    Ok(())
}
*/

// Kicka un utente
async fn kick_user(cx: &Cx) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cx.update.reply_to_message() {
        Some(mes) => {
            // bot.unban_chat_member can also kicks a user from a group chat.
            cx.requester
                .unban_chat_member(cx.update.chat_id(), mes.from().unwrap().id)
                .send()
                .await?;
        }
        None => {
            cx.reply_to("Usa questo comando in risposta ad un messaggio").send().await?;
        }
    }
    Ok(())
}

async fn answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Commands,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {

        Commands::Help => cx.answer(Commands::descriptions()).await?,

        Commands::Username(username)             => {
            cx.answer(format!("Your username is @{}.", username)).await?
        }

        Commands::UsernameAndAge{username, age}  => {
            cx.answer(format!("Your username is @{} and age is {}.", username, age)).await?
        }

        Commands::Kick                           => {
            kick_user(&cx).await?;
            cx.answer(format!("Utente Kickato")).await?
        }

        Commands::Macro{option, macro_str}       => {
            
            match option.as_str() {
            
                "-a" | "--add"                  => {
                    cx.answer(format!("Macro {} aggiunta con opzione {}", macro_str, option)).await?
                },

                "-e" | "--edit"                 => {
                    cx.answer(format!("Macro editata")).await?
                },

                "-r" | "--remove"               => {
                    cx.answer(format!("Macro rimossa")).await?
                }

                "-c" | "--to-ascii"              => {
                    
                    let mut cmd = Command::new("sh");
                    let j = ["echo", macro_str.as_str()].join(" ");
                    cmd.arg("-c").arg(j);
                    let _cmd = cmd.output().expect("Comando non letto correttamente");
                         
                    cx.answer(format!("{:?}", _cmd.stdout)).await?

                }

                _                               =>  cx.answer("Comando non valido").await? // Wildcard
            
            }
        }

        Commands::Ban(stringa)                   => {
            cx.answer(format!("Sei stato bannato {}",stringa)).await?
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting simple_commands_bot...");

    let _bot = Bot::from_env().auto_send();

    let Me {user: _bot_user, ..} = _bot.get_me().await.unwrap();
//    let _bot_name: String = panic!("hacktoberBot");
    let _bot_name: String = "hacktoberBot".into();
    teloxide::commands_repl(_bot, _bot_name, answer).await;
}
