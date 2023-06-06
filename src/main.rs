use serenity::async_trait;
use serenity::framework::standard::StandardFramework;
use serenity::model::prelude::interaction::Interaction;
use serenity::model::prelude::Ready;
use serenity::prelude::{Client, Context, EventHandler, GatewayIntents};

pub mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, _ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => commands::handle_command(command),
            other => eprintln!("{:?} not supported", other),
        }
    }

    async fn ready(&self, _ctx: Context, bot: Ready) {
        println!("Connected as {}#{}", bot.user.name, bot.user.discriminator);

        // Retrieve and print the bot's invite URL
        let invite_url = format!(
            "https://discord.com/oauth2/authorize?client_id={}&scope=bot",
            bot.user.id
        );
        println!("Invite URL: {}", invite_url);
    }
}

#[tokio::main]
async fn main() {
    // set the bot's prefix to "/"
    let framework = StandardFramework::new().configure(|c| c.prefix("/"));

    // Login with a bot token from the environment
    let token = "MTExMDAzMDQyNzg2OTE1MTMzNA.Gx4tiA.RmV2RPFeiIF8lEg9x2THGjapRGzC52UodHA9UY";
    let application_id = 1110030427869151334;

    let intents = GatewayIntents::default();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    tokio::task::block_in_place(|| {
        commands::create_commands(&client);
    });

    // start listening for events by starting a single shard
    if let Err(err) = client.start().await {
        panic!("An error occurred while running the client: {:?}", err);
    }
}
