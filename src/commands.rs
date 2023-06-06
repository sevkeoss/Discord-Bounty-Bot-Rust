use serenity::model::prelude::command::{self, CommandOptionType};
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandData,
};
use serenity::{http, Client};
use tokio::runtime::Runtime;

use std::sync;

pub mod bounty;

enum Commands {
    Bounty,
    Unknown,
}

pub fn create_commands(client: &Client) {
    let http: sync::Arc<http::Http> = client.cache_and_http.http.clone();

    let rt = Runtime::new().expect("Could not create tokio Runtime object.");

    rt.block_on(command::Command::create_global_application_command(
        &http,
        |command| {
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
                        .required(true)
                })
        },
    ))
    .expect("Could not register bounty command.");
}

pub fn handle_command(command: ApplicationCommandInteraction) {
    let command_type = convert_command(&command.data);

    match command_type {
        Commands::Bounty => {
            bounty::handle_bounty(command);
        }
        _ => eprint!("Command not supported"),
    }
}

fn convert_command(command_data: &CommandData) -> Commands {
    match command_data.name.as_str() {
        "bounty" => Commands::Bounty,
        _ => Commands::Unknown,
    }
}
