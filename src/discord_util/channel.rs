use serenity::http::Http;
use serenity::model::permissions::Permissions;

use serenity::model::prelude::{
    ChannelId, ChannelType, GuildChannel, GuildId, PermissionOverwrite, PermissionOverwriteType,
    UserId,
};
use serenity::model::user::User;
use serenity::prelude::Context;

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

pub async fn create_private_text_channel(
    http: &Http,
    guild_id: GuildId,
    category_name: &str,
    lister: User,
    hunter: User,
) {
    let channel_name = "Private Channel";

    println!("In here");
    let category_id: Option<ChannelId> = get_category_id(http, guild_id, category_name).await;
    if let None = category_id {
        println!("Not found");
        return;
    }

    let everyone_role = guild_id
        .roles(http)
        .await
        .unwrap()
        .values()
        .find(|&role| role.name == "@everyone")
        .unwrap()
        .clone();

    println!("Everyone role done");

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
            println!("Channel: {:?}", channel.name());
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
