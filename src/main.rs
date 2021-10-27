//#![allow(unused_imports)]
use teloxide::{prelude::*, types::{ChatPermissions, Me}, utils::command::BotCommand};
use std::env;
use std::error::Error;
use std::str;
use std::str::FromStr;
use std::process::Command;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};

//use teloxide::types::ChatMemberKind;
//use Message::Kind;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Commands {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a macro.", parse_with = "split")]
    Macro {option: String, macro_str: String},
    #[command(description = "ban a user.")]
    Ban, 
    #[command(description = "kick a user.")]
    Kick,
    #[command(description = "mute a user.", parse_with = "split")]    
    Mute {time: u64, unit: UnitOfTime},
    #[command(description = "unban a user.")]
    Unban,
}

enum UnitOfTime {
    Seconds,
    Minutes,
    Hours,
}

impl FromStr for UnitOfTime {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "h" | "hours" => Ok(UnitOfTime::Hours),
            "m" | "minutes" => Ok(UnitOfTime::Minutes),
            "s" | "seconds" => Ok(UnitOfTime::Seconds),
            _ => Err("Allowed units: h, m, s"),
        }
    }
}

// Calculates time of user restriction.
fn calc_restrict_time(time: u64, unit: UnitOfTime) -> Duration {
    match unit {
        UnitOfTime::Hours => Duration::hours(time as i64),
        UnitOfTime::Minutes => Duration::minutes(time as i64),
        UnitOfTime::Seconds => Duration::seconds(time as i64),
    }
}


type Cx = UpdateWithCx<AutoSend<Bot>, Message>;

//Muta un utente rispondendo a un suo messaggio
//Aggiungere opzione che muta per un numero n random di ore [FACOLTATIVO]
//Aggiungere welcome_message e macro personalizzabili

async fn mute_user(cx: &Cx, time: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cx.update.reply_to_message() {
        Some(msg1) => {
            
            // Bot::from_env().get_chat_member(cx.update.chat_id(), msg1.from().unwrap().id).send().await?;
            //let user_id = is_ok!(msg1.update.from()).id;

            let member = cx.requester.get_chat_member(cx.update.chat_id(), msg1.from().unwrap().id).send().await?;
            let _member = member.is_privileged();
            match _member {
                true => {
                    cx.reply_to("true").send().await?;
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

                false => {
                    cx.reply_to("false").send().await?;
                }
            }
            
            /*
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
            */

        }
        None => {
            cx.reply_to("Devi usare questo comando in risposta a un messaggio").send().await?;
        }
    }
    Ok(())
}


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

// Banna un utente 
async fn ban_user(cx: &Cx) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cx.update.reply_to_message() {
        Some(message) => {
            cx.requester
                .kick_chat_member(
                    cx.update.chat_id(),
                    message.from().expect("Must be MessageKind::Common").id,
                ).await?;
        }
        None => {
            cx.reply_to("Usa questo comando in risposta ad un messaggio").send().await?;
        }
    }
    Ok(())
}

/*
async fn check_user(cx: &Cx) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cx.update.reply_to_message() {
        Some(msge) => {
                is_banned(
                    cx.update.chat_id()
                    ).await?;
        }
        None => {
            cx.reply_to("Usa questo comando in risposta ad un messaggio").send().await?;
        }
    }
    Ok(())
}*/

async fn answer(cx: UpdateWithCx<AutoSend<Bot>, Message>, command: Commands,) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {

        Commands::Help => cx.answer(Commands::descriptions()).await?,
        
        /*
        Commands::Isbanned                       => {
            check_user(&cx).await?;
            cx.answer(format!("BRUH")).await?
        }*/

        Commands::Unban                          => {
            kick_user(&cx).await?;
            cx.answer(format!("Utente sbannato")).await?
        }

        Commands::Ban                            => {
            ban_user(&cx).await?;
            cx.answer(format!("Utente bannato")).await?
        }
        
        Commands::Kick                           => {
            kick_user(&cx).await?;
            cx.answer(format!("Utente kickato")).await?
        }

        
        Commands::Mute{time, unit}               => {
            mute_user(&cx, calc_restrict_time(time, unit)).await?;
            cx.answer(format!("Utente mutato")).await?
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
    
    let _bot_name: String = "INIT.D".into();
    teloxide::commands_repl(_bot, _bot_name, answer).await;
}
