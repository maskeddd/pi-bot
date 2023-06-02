use std::{io::Write, process::Command};

use tempfile::NamedTempFile;

use crate::{Context, Error};

#[poise::command(prefix_command, slash_command, subcommands("upload"))]
pub async fn dropbox(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn upload(
    ctx: Context<'_>,
    #[description = "The file to upload"] file: poise::serenity_prelude::Attachment,
) -> Result<(), Error> {
    ctx.defer().await?;

    let buffer = file.download().await?;
    let mut tmpfile = NamedTempFile::new()?;
    tmpfile.write_all(&buffer)?;

    let path = tmpfile.into_temp_path();

    let output = Command::new("./dropbox_uploader.sh")
        .arg("upload")
        .arg(path.display().to_string())
        .arg(file.filename)
        .output()?;

    if output.status.success() {
        ctx.say("File uploaded successfully").await?;
    } else {
        ctx.say("Failed to upload file").await?;
    }

    path.close()?;

    Ok(())
}
