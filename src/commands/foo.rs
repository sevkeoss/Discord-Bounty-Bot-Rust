use serenity::http::Http;
use serenity::model::permissions::Permissions;
use serenity::model::prelude::command::{self, CommandOptionType};
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandData,
};

use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::{
    ChannelId, ChannelType, GuildChannel, GuildId, PermissionOverwrite, PermissionOverwriteType,
    UserId,
};
use serenity::model::user::User;
use serenity::prelude::Context;
use serenity::{http, Client};
use tokio::runtime::Runtime;

use std::sync;

pub mod bounty;

enum CommandType {
    Bounty,
    Unknown,
}

pub async fn confirm_bounty_listing(ctx: &Context, command: &ApplicationCommandInteraction) {
    // Process the slash command and generate the response message
    let response_message = "Hello, this is the reply to your slash command!";

    println!("In func");
    // Send the response message with the action row
    if let Err(why) = command
        .create_interaction_response(
            &ctx.http,
            |r: &mut serenity::builder::CreateInteractionResponse| {
                r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| {
                        d.content(response_message).components(|c| {
                            c.create_action_row(|row| {
                                row.create_button(|b| {
                                    b.style(ButtonStyle::Success)
                                        .label("Confirm")
                                        .custom_id("button_click")
                                })
                            })
                        })
                    })
            },
        )
        .await
    {
        println!("Failed to send reply: {:?}", why);
    }

    println!("Finishing");
}

pub fn handle_command(ctx: &Context, http: &Http, command: ApplicationCommandInteraction) {
    let command_type = convert_command(&command.data);

    let rt = Runtime::new().expect("Could not create tokio Runtime object.");

    match command_type {
        CommandType::Bounty => {
            unsafe { rt.block_on(bounty::handle_bounty(ctx, http, command)) };
        }
        _ => eprint!("Command not supported"),
    }
}

fn convert_command(command_data: &CommandData) -> CommandType {
    match command_data.name.as_str() {
        "bounty" => CommandType::Bounty,
        _ => CommandType::Unknown,
    }
}

pub async fn create_category_if_no_exist(http: &Http, guild_id: GuildId, category_name: &str) {
    let category_id = get_category_id(http, guild_id, category_name).await;
    if let Some(_) = category_id {
        return;
    }

    let result: Result<GuildChannel, serenity::Error> = guild_id
        .create_channel(http, |c| c.name(category_name).kind(ChannelType::Category))
        .await;

    if let Err(err) = result {
        eprintln!("Error creating category: {:?}", err);
    } else {
        println!("Successfully created category: {}", category_name);
    }
}

async fn create_private_text_channel(
    http: &Http,
    guild_id: GuildId,
    category_name: &str,
    lister: User,
    hunter: User,
) {
    let channel_name = "Private Channel";

    let category_id: Option<ChannelId> = get_category_id(http, guild_id, category_name).await;
    if let None = category_id {
        return;
    }

    let bot_id = http.get_current_user().await.unwrap().id;

    let everyone_role = guild_id
        .roles(http)
        .await
        .unwrap()
        .values()
        .find(|&role| role.name == "@everyone")
        .unwrap()
        .clone();

    match guild_id
        .create_channel(http, |channel| {
            channel
                .name(channel_name)
                .kind(ChannelType::Text)
                .category(category_id.unwrap())
                .permissions(vec![
                    PermissionOverwrite {
                        allow: Permissions::empty(),
                        deny: Permissions::VIEW_CHANNEL,
                        kind: PermissionOverwriteType::Role(everyone_role.id), // User ID of the specific user
                    },
                    PermissionOverwrite {
                        allow: Permissions::VIEW_CHANNEL,
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Member(UserId(1110030427869151334)),
                    },
                    PermissionOverwrite {
                        allow: Permissions::VIEW_CHANNEL,
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Member(lister.id), // User ID of the specific user
                    },
                    PermissionOverwrite {
                        allow: Permissions::VIEW_CHANNEL,
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Member(hunter.id), // User ID of the specific user
                    },
                    // Add more PermissionOverwrite objects for other users or roles as needed
                ])
        })
        .await
    {
        Ok(_) => (),
        Err(err) => eprintln!("Could not create private channel. {}", err),
    };
}

async fn get_category_id(http: &Http, guild_id: GuildId, category_name: &str) -> Option<ChannelId> {
    let channels = guild_id.channels(http).await;

    if let Ok(channels) = channels {
        for channel in channels.values() {
            if channel.kind == ChannelType::Category && channel.name == category_name {
                return Some(channel.id);
            }
        }
    }

    None
}

async fn get_bot_permissions(ctx: &Context, guild_id: GuildId) {
    if let Ok(bot_member) = guild_id
        .member(&ctx.http, ctx.cache.current_user_id())
        .await
    {
        println!("Bot member: {:?}", bot_member);
        if let Ok(permissions) = bot_member.permissions(&ctx.cache) {
            println!("Permissions: {}", permissions);
        } else {
            println!("Not ok");
        }
    }
}
