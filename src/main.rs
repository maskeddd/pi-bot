mod commands;

use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity, GuildId};
use serde::Deserialize;
use std::{env::var, fs::File, io::BufReader, path::Path};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Deserialize, Debug, Clone)]
pub struct City {
    pub name: String,
    pub lon: String,
    pub lat: String,
    pub country: String,
}

pub struct Data {
    pub cities: Vec<City>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::ping::ping(),
            commands::weather::weather(),
            commands::info::info(),
            commands::dropbox::dropbox(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            ..Default::default()
        },

        on_error: |error| Box::pin(on_error(error)),

        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },

        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },

        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),

        skip_checks_for_owners: false,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!("Got an event in event handler: {:?}", event.name());
                Ok(())
            })
        },
        ..Default::default()
    };

    poise::Framework::builder()
        .token(var("DISCORD_TOKEN").expect("Missing `DISCORD_TOKEN` env var."))
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId(
                        var("GUILD_ID")
                            .expect("Missing `GUILD_ID` env var.")
                            .parse()
                            .unwrap(),
                    ),
                )
                .await?;

                let file_path = Path::new("data/cities.json");
                let file = File::open(file_path)?;
                let reader = BufReader::new(file);
                let cities: Vec<City> = serde_json::from_reader(reader)?;

                Ok(Data { cities })
            })
        })
        .options(options)
        .intents(serenity::GatewayIntents::non_privileged())
        .run_autosharded()
        .await
        .unwrap();
}
