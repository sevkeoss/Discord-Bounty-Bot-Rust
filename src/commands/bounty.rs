use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    io::{self, Error},
};

use serenity::{
    builder::{CreateApplicationCommand, CreateInteractionResponse},
    http::Http,
    model::{
        prelude::{
            command::CommandOptionType,
            component::ButtonStyle,
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
                message_component::MessageComponentInteraction,
                InteractionResponseType, MessageFlags,
            },
        },
        user::User,
    },
};
use uuid::Uuid;

use crate::discord_util;

static mut ACTIVE_BOUNTIES: Lazy<HashMap<Uuid, Bounty>> = Lazy::new(|| {
    let map = HashMap::new();
    map
});

#[derive(Debug)]
struct Bounty {
    lister: User,
    hunter: User,
    bounty_number: u32,
    middlemen: Vec<User>,
}

impl Bounty {
    pub fn new(lister: User, hunter: User, bounty_number: u32) -> Bounty {
        Bounty {
            lister: lister.clone(),
            hunter,
            bounty_number,
            middlemen: Vec::new(),
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("bounty")
        .description("Start a bounty with the specified bounty hunter")
        .create_option(|option| {
            option
                .name("hunter")
                .description("The bounty hunter")
                .kind(CommandOptionType::User)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("number")
                .description("The bounty number")
                .kind(CommandOptionType::Integer)
                .min_int_value(1)
                .required(true)
        })
}

pub fn run<'a>(command: &ApplicationCommandInteraction) -> CreateInteractionResponse<'a> {
    let message: String = "Please confirm the bounty".to_string();
    let id = Uuid::new_v4();

    let (hunter, number) = extract_command_args(command.clone());

    let new_bounty = Bounty::new(command.user.clone(), hunter, number);
    unsafe {
        ACTIVE_BOUNTIES.insert(id, new_bounty);
    }

    CreateInteractionResponse::default()
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|d| {
            d.content(message)
                .flags(MessageFlags::EPHEMERAL)
                .components(|c| {
                    c.create_action_row(|row| {
                        row.create_button(|b| {
                            b.style(ButtonStyle::Success)
                                .label("Confirm Bounty")
                                .custom_id(String::from("bounty/") + id.to_string().as_str())
                        })
                    })
                })
        })
        .clone()
}

pub async fn confirm_bounty(
    http: &Http,
    component: &MessageComponentInteraction,
) -> Result<(), String> {
    let ind = component.data.custom_id.find('/').unwrap();
    let id = &component.data.custom_id[ind + 1..];

    let curr_bounty;
    unsafe {
        curr_bounty = ACTIVE_BOUNTIES.get(&Uuid::parse_str(id).unwrap());
    }

    match curr_bounty {
        Some(bounty) => {
            if component.user != bounty.lister {
                return Err(String::from(
                    "Only the bounty lister can confirm the command",
                ));
            }

            println!("Before create");
            discord_util::channel::create_private_text_channel(
                http,
                component.guild_id.unwrap(),
                "BOUNTY PLATFORM",
                bounty.lister.clone(),
                bounty.hunter.clone(),
            )
            .await;
        }
        None => {
            return Err(String::from("Not found"));
        }
    }

    Ok(())
}

fn extract_command_args(input: ApplicationCommandInteraction) -> (User, u32) {
    let mut hunter: User = User::default();
    let mut number: u32 = 0;
    for arg in input.data.options {
        match arg.name.as_str() {
            "hunter" => {
                if let Some(arg) = arg.resolved {
                    if let CommandDataOptionValue::User(user, _) = arg {
                        hunter = user.clone();
                    }
                }
            }
            "number" => {
                if let Some(arg) = arg.resolved {
                    if let CommandDataOptionValue::Integer(val) = arg {
                        if val <= u32::MAX as i64 {
                            number = val as u32;
                        } else {
                            eprintln!("Invalid bounty number");
                        }
                    }
                }
            }
            _ => {
                eprintln!("Unknown argument");
            }
        }
    }

    (hunter, number)

    // super::get_bot_permissions(ctx, guild_id).await;

    // super::create_private_text_channel(
    //     http,
    //     guild_id,
    //     "BOUNTY PLATFORM",
    //     input.user.clone(),
    //     hunter.clone(),
    // )
    // .await;
}
