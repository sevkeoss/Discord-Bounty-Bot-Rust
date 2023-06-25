use dotenv::dotenv;
use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::model::prelude::{GuildId, Ready};
use serenity::prelude::{Client, Context, EventHandler, GatewayIntents};

use std::env;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "bounty" => commands::bounty::run(&command.data.options),
                _ => "not implemented".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(
                    &ctx.http,
                    |r: &mut serenity::builder::CreateInteractionResponse| {
                        r.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| d.content(content).components(|c| c))
                    },
                )
                .await
            {
                println!("Failed to send reply: {:?}", why);
            }
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

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| commands::bounty::register(command))
        })
        .await;

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
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
