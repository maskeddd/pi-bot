use crate::{Context, Error};
use std::time::Instant;

/// Gets roundtrip latency
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();
    let msg = ctx.say("Pong!").await?;
    let elapsed = Instant::now().duration_since(start).as_millis();

    msg.edit(ctx, |m| {
        m.content(format!(":ping_pong: Pong! `{}ms`", elapsed))
    })
    .await?;

    Ok(())
}
