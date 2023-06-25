use once_cell::sync::Lazy;
use std::{collections::HashMap, env};

use serenity::{
    builder::CreateApplicationCommand,
    http::Http,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::application_command::{
                ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
            },
            GuildId,
        },
        user::User,
    },
    prelude::Context,
};
use uuid::Uuid;

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
            lister,
            hunter,
            bounty_number,
            middlemen: Vec::new(),
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("bounty")
        .description("Testing bounty bot")
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

pub fn run(options: &[CommandDataOption]) -> String {
    println!("Options {:?}", options);
    "Hey, I'm alive!".to_string()

    // c.create_action_row(|row| {
    //     row.create_button(|b| {
    //         b.style(ButtonStyle::Success)
    //             .label("Confirm")
    //             .custom_id("button_click")
    //     })
    // })
}

pub async unsafe fn handle_bounty(
    ctx: &Context,
    http: &Http,
    input: ApplicationCommandInteraction,
) {
    let id = Uuid::new_v4();

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
                        if val >= 0 && val <= u32::MAX as i64 {
                            number = val as u32;
                        } else {
                            eprintln!("Invalid bounty number");
                            return;
                        }
                    }
                }
            }
            _ => {
                eprintln!("Unknown argument");
            }
        }
    }

    let new_bounty = Bounty::new(input.user.clone(), hunter.clone(), number);
    ACTIVE_BOUNTIES.insert(id, new_bounty);

    let guild_id = GuildId(
        env::var("GUILD_ID")
            .expect("GUILD_ID not set.")
            .parse()
            .expect("Could not parse GUILD_ID"),
    );

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
