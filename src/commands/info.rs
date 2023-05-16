use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Context, Error};
use serenity::{builder::CreateEmbed, http::CacheHttp};
use systemstat::{saturating_sub_bytes, Duration, Platform, System};

/// Gets info about the pi
#[poise::command(slash_command)]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let system = System::new();

    let mut embed = CreateEmbed::default();
    embed
        .colour(0xe10054)
        .title("pi Info")
        .thumbnail(ctx.cache().unwrap().current_user().avatar_url().unwrap());

    if let Ok(uptime) = system.uptime() {
        let unix_time = SystemTime::now().duration_since(UNIX_EPOCH)?;
        embed.field(
            "Uptime",
            format!("<t:{}:R>", (unix_time - uptime).as_secs()),
            false,
        );
    }

    if let Ok(mem) = system.memory() {
        embed.field(
            "Memory",
            format!(
                "{} used / {} total",
                saturating_sub_bytes(mem.total, mem.free),
                mem.total
            ),
            false,
        );
    }

    if let Ok(cpu) = system.cpu_load_aggregate() {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let cpu = cpu.done().unwrap();
        embed.field(
            "CPU Load",
            format!(
                "System: {}%\nUser: {}%\nIdle: {}%",
                cpu.user * 100.,
                cpu.system * 100.,
                cpu.idle * 100.
            ),
            false,
        );
    }

    if let Ok(cpu_temp) = system.cpu_temp() {
        embed.field("CPU Temperature", cpu_temp, false);
    }

    if let Ok(disk) = system.mount_at("/") {
        embed.field(
            "Disk Usage",
            format!("{} used / {} total", disk.avail, disk.total),
            false,
        );
    }

    ctx.send(|m| {
        m.embed(|e| {
            *e = embed;
            e
        })
    })
    .await?;

    Ok(())
}
