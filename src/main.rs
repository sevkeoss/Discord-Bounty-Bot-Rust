use dotenv::dotenv;
use serenity::async_trait;
use serenity::builder::CreateInteractionResponse;
use serenity::framework::StandardFramework;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::{GuildId, Ready};
use serenity::prelude::{Client, Context, EventHandler, GatewayIntents};

use std::env;

mod commands;
mod discord_util;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let content = match command.data.name.as_str() {
                    "bounty" => commands::bounty::run(&command),
                    _ => CreateInteractionResponse::default()
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .clone(),
                };

                if let Err(err) = command
                    .create_interaction_response(&ctx.http, |r| {
                        *r = content.clone();
                        r
                    })
                    .await
                {
                    eprintln!("Failed to ask for confirmation: {:?}", err);
                }
            }
            Interaction::MessageComponent(component) => {
                let ind = component
                    .data
                    .custom_id
                    .find('/')
                    .unwrap_or_else(|| component.data.custom_id.len());
                let command = &component.data.custom_id[..ind];
                let id = &component.data.custom_id[ind + 1..];
                match command {
                    "bounty" => {
                        let res = commands::bounty::confirm_bounty(&ctx.http, &component).await;
                        match res {
                            Ok(_) => {
                                if let Err(err) = component
                                    .create_interaction_response(&ctx.http, |r| {
                                        r.kind(InteractionResponseType::UpdateMessage)
                                            .interaction_response_data(|d| {
                                                d.content("Confirmed").components(|c| c)
                                            })
                                    })
                                    .await
                                {
                                    eprintln!("Failed to confirm bounty: {:?}", err);
                                }
                            }
                            Err(err) => {
                                eprintln!("Err: {}", err);
                            }
                        }
                    }
                    "Accept" => commands::bounty::accept(&ctx.http, &component, id).await,
                    "Decline" => commands::bounty::decline(&ctx.http, &component).await,
                    "Complete" => commands::bounty::complete(&ctx, &component).await,
                    _ => eprintln!("Uknown button id"),
                }
            }
            _ => (),
        }
    }

    async fn ready(&self, ctx: Context, bot: Ready) {
        println!("Connected as {}#{}", bot.user.name, bot.user.discriminator);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("GUILD_ID not set.")
                .parse()
                .expect("Could not parse GUILD_ID"),
        );

        if let Err(err) = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| commands::bounty::register(command))
        })
        .await
        {
            panic!("Could not register commands. {}", err);
        };

        let category_name: String = env::var("BOUNTY_CATEGORY").expect("Bounty Category not set.");

        discord_util::channel::create_category_if_no_exist(&ctx.http, guild_id, &category_name)
            .await;
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set.");

    let framework = StandardFramework::new().configure(|c| c.prefix("/"));

    let intents = GatewayIntents::default();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(err) = client.start().await {
        panic!("An error occurred while starting the client: {:?}", err);
    }
}
