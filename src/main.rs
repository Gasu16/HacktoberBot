//#![allow(unused_imports)]
// Last edit: 22:21 - 31/10/2021
use teloxide::{prelude::*, types::{ChatPermissions, Me}, utils::command::BotCommand};
use std::env;
use std::error::Error;
use std::str;
use std::str::FromStr;
use std::process::Command;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "Lista comandi", parse_with = "split")]
enum Commands {
    #[command(description = "Mostra lista comandi.")]
    Help,
    #[command(description = "Gestisci una macro.", parse_with = "split")]
    Macro {option: String, macro_str: String},
    #[command(description = "Banna un utente da un gruppo.")]
    Ban, 
    #[command(description = "Kicka un utente da un gruppo.")]
    Kick,
    #[command(description = "Muta un utente da un gruppo.", parse_with = "split")]    
    Mute {time: u64, unit: UnitOfTime},
    #[command(description = "Annulla il ban ad un utente di un gruppo.")]
    Unban,
    #[command(description = "Controlla i log.")]
    Logs,
    #[command(description = "Il bot risponde?")]
    On,
    #[command(description = "La mia pagina github.")]
    Info,
    #[command(description = "Effettua un calcolo.")]
    Calc {x: u32, y: u32, operator: String},
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

/*
async fn Calc(cx: &Cx, x: u64, y: u64, operator: &str) { 
    match operator {
        "+" => cx.answer(format!("{}", x+y)),
        "-" => cx.answer(format!("{}", x-y)),
        "x" => cx.answer(format!("{}", x*y)),
        "/" => cx.answer(format!("{}", x/y)),
        _   => cx.answer("Non ho capito che operazione vuoi fare"),
    };
}
*/

//Muta un utente rispondendo a un suo messaggio
//Aggiungere welcome_message e macro personalizzabili

async fn mute_user(cx: &Cx, time: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cx.update.reply_to_message() {

        Some(msg1) => {

            // Controlliamo i permessi di chi invoca il comando
            // Che ruolo ha? Amministratore o utente

            let member_mute = cx.requester.get_chat_member(cx.update.chat_id(), cx.update.from().unwrap().id).send().await?;
            
            let _member_mute = member_mute.is_privileged();

            match _member_mute {

                true => {

                    // Se entra qui dentro vuol dire che un amministratore sta invocando il comando
                    // Utente che "subisce" il comando
                    // controlliamo se e' un admin o un normale utente

                    let to_mute = cx.requester.get_chat_member(cx.update.chat_id(), msg1.from().unwrap().id).send().await?;
                    let _to_mute = to_mute.is_privileged();

                    match _to_mute {
                        
                        // L'utente che "subisce" il comando e' un amministratore
                        true => {
                            cx.reply_to("Non posso usare questo comando su un amministratore").send().await?;
                        }

                        // L'utente che "subisce" il comando e' un utente normale
                        // Si puo' procedere con il comando
                        false => {                    
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
                            cx.answer(format!("{} e' stato mutato fino al {}", msg1.from().unwrap().first_name, DateTime::<Utc>::from_utc(
                                        NaiveDateTime::from_timestamp(cx.update.date as i64, 0),
                                        Utc,
                                    ) + time)).await?;
                        }
                    }
                }

                false => {

                    // Un utente normale sta cercando di usare un comando per soli amministratori

                    cx.reply_to("Non hai i permessi necessari per usare questo comando").send().await?;
                }
            }
        }

        None => {
            // Non viene specificato nessun messaggio a cui rispondere

            cx.reply_to("Usa questo comando in risposta ad un messaggio").send().await?;
        }
    }
    Ok(())
}


// Kicka un utente
async fn kick_user(cx: &Cx, str_msg: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cx.update.reply_to_message() {

        Some(mes) => {

            // Controlliamo i permessi di chi invoca il comando
            // Che ruolo ha? Amministratore o utente

            let member_kick = cx.requester.get_chat_member(cx.update.chat_id(), cx.update.from().unwrap().id).send().await?;
            
            let _member_kick = member_kick.is_privileged();
            
            match _member_kick {

                true => {
                    // Se entra qui dentro vuol dire che un amministratore sta invocando il comando
                    // Utente che "subisce" il comando
                    // controlliamo se e' un admin o un normale utente

                    let to_kick = cx.requester.get_chat_member(cx.update.chat_id(), mes.from().unwrap().id).send().await?;
                    let _to_kick = to_kick.is_privileged();
                    
                    match _to_kick {
                        
                        // L'utente che "subisce" il comando e' un amministratore
                        true => {

                            cx.reply_to("Non posso usare questo comando su un amministratore").send().await?;
                        }
                        
                        // L'utente che "subisce" il comando e' un utente normale
                        // Si puo' procedere con il comando
                        false => {
                            cx.requester
                                .unban_chat_member(cx.update.chat_id(), mes.from().unwrap().id)
                                .send()
                                .await?;
                            cx.answer(format!("{} {}", mes.from().unwrap().first_name, str_msg)).await?;
                            //cx.answer(format!("Utente {} kickato", mes.from().unwrap().id)).await?;
                        }
                    }
                }

                false => {

                    // Un utente normale sta cercando di usare un comando per soli amministratori

                    cx.reply_to("Non hai i permessi necessari per usare questo comando").send().await?;
                }
            }

        }
        None => {
            // Non viene specificato nessun messaggio a cui rispondere
            
            cx.reply_to("Usa questo comando in risposta ad un messaggio").send().await?;
        }
    }
    Ok(())
}

// Banna un utente 
async fn ban_user(cx: &Cx) -> Result<(), Box<dyn Error + Send + Sync>> {
    match cx.update.reply_to_message() {
        Some(message) => {
            
            // Controlliamo i permessi di chi invoca il comando
            // Che ruolo ha? Amministratore o utente

            let member_ban = cx.requester.get_chat_member(cx.update.chat_id(), cx.update.from().unwrap().id).send().await?;
            
            let _member_ban = member_ban.is_privileged();

            match _member_ban {
                
                true => { 

                    // Se entra qui dentro vuol dire che un amministratore sta invocando il comando
                    // Utente che "subisce" il comando
                    // controlliamo se e' un admin o un normale utente

                    let to_ban = cx.requester.get_chat_member(cx.update.chat_id(), message.from().unwrap().id).send().await?;
                    let _to_ban= to_ban.is_privileged();

                    match _to_ban {
            
                        // L'utente che "subisce" il comando e' un amministratore
                        true => {
                            cx.reply_to("Non posso usare questo comando su un amministratore").send().await?;
                        }
                          
                        // L'utente che "subisce" il comando e' un utente normale
                        // Si puo' procedere con il comando
                        false => {
               
                            cx.requester
                                .kick_chat_member(
                                    cx.update.chat_id(),
                                    message.from().expect("Must be MessageKind::Common").id,
                                ).await?;
                            cx.answer(format!("{} e' stato bannato", message.from().unwrap().first_name)).await?;
                        }
                    }
                } 

                false => {
                    cx.reply_to("Non hai i permessi necessari per usare questo comando").send().await?;
                }
            }
        }
        None => {
            cx.reply_to("Usa questo comando in risposta ad un messaggio").send().await?;
        }
    }
    Ok(())
}

async fn action(cx: UpdateWithCx<AutoSend<Bot>, Message>, command: Commands) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {

        Commands::Help                           => {
            print_(&cx, &Commands::descriptions()).await?;
        }

        Commands::Logs                           => {
            print_(&cx, "@rootinit controlla i log").await?;
        }

        Commands::On                             => {
            print_(&cx, "Sono online...").await?;
        }

        Commands::Calc{x, y, operator}           => {
            match operator.as_str() {
                "+" | "add"                      => {
                    let a = x.checked_add(y);
                    match a {
                        Some(_v_add) => {
                            print_op(&cx, "", x+y).await?;
                        }
                        
                        None => {
                            print_(&cx, "Ops, non sono stato in grado di effettuare il calcolo, riprova").await?;
                        }
                    };

                }

                "-" | "sub"                      => {
                    let s = x.checked_sub(y);
                    match s {
                        Some(_v_sub) => {
                            print_op(&cx, "", x-y).await?;
                        }
                        
                        None => {
                            print_(&cx, "Ops, non sono stato in grado di effettuare il calcolo, riprova").await?;
                        }
                    };

                }

                "x" | "mul"                      => {
                    let m = x.checked_add(y);
                    match m {
                        Some(_v_mul) => {
                            print_op(&cx, "", x*y).await?;
                        }
                        
                        None => {
                            print_(&cx, "Ops, non sono stato in grado di effettuare il calcolo, riprova").await?;
                        }
                    };

                }

                "/" | "div"                      => {
                    let d = x.checked_add(y);
                    match d {
                        Some(_v_div) => {
                            print_op(&cx, "", x/y).await?;
                        }
                        
                        None => {
                            print_(&cx, "Ops, non sono stato in grado di effettuare il calcolo, riprova").await?;
                        }
                    };

                }

                "**"| "pow"                      => {
                    //let p = x.pow(y);
                    let p = x.checked_pow(y);
                    match p {
                        Some(_v) => {
                            print_op(&cx, "", x.pow(y)).await?;
                        }
                        
                        None => {
                            print_(&cx, "Ops, non sono stato in grado di effettuare il calcolo, riprova").await?;
                        }
                    };
                }

                _                                => {
                    print_(&cx, "Non ho capito che operazione devo fare ").await?;
                }
            }
        }

        Commands::Info                           => {
            print_(&cx, "https://github.com/Gasu16/HacktoberBot").await?;    
        }

        Commands::Unban                          => {
            kick_user(&cx, "e' stato sbannato").await?;
        }

        Commands::Ban                            => {
            ban_user(&cx).await?;
        }
        
        Commands::Kick                           => {           
            kick_user(&cx, "e' stato kickato").await?;
        }

        
        Commands::Mute{time, unit}               => {
            mute_user(&cx, calc_restrict_time(time, unit)).await?;
        }

        Commands::Macro{option, macro_str}       => {
            
            match option.as_str() {
            
                "-a" | "--add"                   => {
                    print_(&cx, "Macro aggiunta").await?;
                }

                "-e" | "--edit"                  => {
                    print_(&cx, "Macro editata").await?;
                }

                "-r" | "--remove"                => {
                    print_(&cx, "Macro rimossa").await?;
                }

                "-c" | "--to-ascii"              => {
                    
                    let mut cmd = Command::new("sh");
                    let j = ["echo", macro_str.as_str()].join(" ");
                    cmd.arg("-c").arg(j);
                    let _cmd = cmd.output().expect("Comando non letto correttamente");
                    print_with(&cx, "", _cmd.stdout).await?;

                }

                _                                => {
                    print_(&cx, "Comando non valido").await?;
                }
            }
        }

    };

    Ok(())
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn print_(cx: &Cx, to_print: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Err(e) = cx.answer(format!("{}", to_print)).await {
        println!("Error: {}", e.to_string());
    }
    Ok(())
}

async fn print_with(cx: &Cx, to_print_with: &str, to_arg_with: Vec<u8>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Err(er) = cx.answer(format!("{} {:?}", to_print_with, to_arg_with)).await {
        println!("Error: {}", er.to_string());
    }
    Ok(())
}

async fn print_op(cx: &Cx, to_print_op: &str, to_arg_op: u32) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Err(op_err) = cx.answer(format!("{} {:?}", to_print_op, to_arg_op)).await {
        println!("Error: {}", op_err.to_string());
    }
    Ok(())
}



async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting simple_commands_bot...");

    let _bot = Bot::from_env().auto_send();
     
    let Me {user: _bot_user, ..} = _bot.get_me().await.unwrap();
    
    let _bot_name: String = "INIT.D".into();
    teloxide::commands_repl(_bot, _bot_name, action).await;
}
