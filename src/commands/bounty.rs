use once_cell::sync::Lazy;
use std::{collections::HashMap, env};

use serenity::{
    builder::{CreateApplicationCommand, CreateInteractionResponse},
    http::{CacheHttp, Http},
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
    prelude::Context,
};
use uuid::Uuid;

use crate::discord_util;

static mut ACTIVE_BOUNTIES: Lazy<HashMap<Uuid, Bounty>> = Lazy::new(|| {
    let map = HashMap::new();
    map
});

#[derive(Debug)]
pub struct Bounty {
    pub lister: User,
    pub hunter: User,
    pub bounty_number: u32,
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

            if let Err(err) = discord_util::channel::create_private_text_channel(
                http,
                component.guild_id.unwrap(),
                "BOUNTY PLATFORM",
                bounty,
                id,
            )
            .await
            {
                eprintln!("Could not create channel: {}", err);
            }
        }
        None => {
            return Err(String::from("Not found"));
        }
    }

    Ok(())
}

pub async fn accept(http: &Http, component: &MessageComponentInteraction, id: &str) {
    let curr_bounty;
    unsafe {
        curr_bounty = ACTIVE_BOUNTIES.get(&Uuid::parse_str(id).unwrap()).unwrap();
    }

    // if component.user != curr_bounty.hunter {
    //     let message = "Only the bounty hunter can accept the bounty";

    //     let _ = component
    //         .create_interaction_response(http, |r| {
    //             r.kind(InteractionResponseType::ChannelMessageWithSource)
    //                 .interaction_response_data(|d| {
    //                     d.content(message).flags(MessageFlags::EPHEMERAL)
    //                 })
    //         })
    //         .await;
    // } else {
    if let Err(err) = component
        .create_interaction_response(http, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| d.content("Accepted").components(|c| c))
        })
        .await
    {
        eprintln!("Failed to accept bounty: {:?}", err);
    }

    let message = "Please complete the bounty when the task is done.";

    let _ = component
        .create_followup_message(http, |m| {
            m.content(message).components(|c| {
                c.create_action_row(|r| {
                    r.create_button(|b| {
                        b.style(ButtonStyle::Success)
                            .label("Complete Bounty")
                            .custom_id(String::from("Complete/") + id)
                    })
                })
            })
        })
        .await;
    //}
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
}

pub async fn decline(http: &Http, component: &MessageComponentInteraction) {
    if let Err(err) = component
        .create_interaction_response(http, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| d.content("Declined").components(|c| c))
        })
        .await
    {
        eprintln!("Failed to decline bounty: {:?}", err);
    }
}

pub async fn complete(ctx: &Context, component: &MessageComponentInteraction) {
    let ni_role = env::var("NI_ROLE").expect("NI Team role name not set.");

    if let Some(member) = &component.member {
        if member.roles.iter().any(|r| {
            if let Some(role) = r.to_role_cached(&ctx.cache) {
                role.name == ni_role
            } else {
                false
            }
        }) {
            if let Err(err) = component
                .create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::UpdateMessage)
                        .interaction_response_data(|d| d.content("Completed").components(|c| c))
                })
                .await
            {
                eprintln!("Failed to complete bounty: {:?}", err);
            }

            discord_util::channel::switch_category(ctx, component, "Archive").await;
        }
    }
}
