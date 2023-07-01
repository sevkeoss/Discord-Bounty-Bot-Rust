use serenity::http::Http;
use serenity::model::permissions::Permissions;

use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::{
    ChannelId, ChannelType, GuildChannel, GuildId, PermissionOverwrite, PermissionOverwriteType,
    UserId,
};

use crate::commands::bounty;

pub async fn create_category_if_no_exist(http: &Http, guild_id: GuildId, category_name: &str) {
    let category_id = get_category_id(http, guild_id, category_name).await;
    if let Some(_) = category_id {
        return;
    }

    let result: Result<GuildChannel, serenity::Error> = guild_id
        .create_channel(http, |c| c.name(category_name).kind(ChannelType::Category))
        .await;

    if let Err(err) = result {
        panic!("Error creating category: {:?}", err);
    }
}

pub async fn create_private_text_channel(
    http: &Http,
    guild_id: GuildId,
    category_name: &str,
    bounty: &bounty::Bounty,
    id: &str,
) -> Result<(), String> {
    let channel_name = format!(
        "{}-{}-bounty{}",
        bounty.lister.name, bounty.hunter.name, bounty.bounty_number
    );

    let category_id: Option<ChannelId> = get_category_id(http, guild_id, category_name).await;
    if let None = category_id {
        return Err(String::from("Could not create private text channel"));
    }

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
                        kind: PermissionOverwriteType::Member(bounty.lister.id), // User ID of the specific user
                    },
                    PermissionOverwrite {
                        allow: Permissions::VIEW_CHANNEL,
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Member(bounty.hunter.id), // User ID of the specific user
                    },
                    // Add more PermissionOverwrite objects for other users or roles as needed
                ])
        })
        .await
    {
        Ok(channel) => {
            let intro = format!(
                "{} would like to start a bounty with you {}. Please accept or decline.",
                bounty.lister.name, bounty.hunter.name
            );

            if let Err(err) = channel
                .send_message(http, |m| {
                    m.content(intro).components(|c| {
                        c.create_action_row(|r| {
                            r.create_button(|b| {
                                b.style(ButtonStyle::Success)
                                    .label("Accept")
                                    .custom_id(String::from("Accept/") + id)
                            })
                            .create_button(|b| {
                                b.style(ButtonStyle::Danger)
                                    .label("Decline")
                                    .custom_id(String::from("Decline/") + id)
                            })
                        })
                    })
                })
                .await
            {
                eprintln!("Could not send intro message for bounty: {}", err);
            };
            Ok(())
        }
        Err(err) => Err(err.to_string()),
    }
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
