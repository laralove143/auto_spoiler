use anyhow::{Context as _, Result};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    guild::{PartialMember, Permissions},
    id::{marker::GuildMarker, Id},
};

use crate::{database, interaction::WordType, Context};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "allow",
    desc = "set what kind of words to allow in the server, or reset the settings"
)]
pub struct Allow {
    #[command(
        name = "word_type",
        desc = "what kind of words do you want to allow, don't set this to reset the settings"
    )]
    word_type: Option<WordType>,
}

pub async fn run(
    ctx: &Context,
    data: CommandData,
    guild_id: Id<GuildMarker>,
    member: PartialMember,
) -> Result<&'static str> {
    if !member
        .permissions
        .context("member in interaction doesn't have permissions")?
        .contains(Permissions::MANAGE_GUILD)
    {
        return Ok("you need the manage guild permission to use this");
    }

    let options = Allow::from_interaction(data.into())?;

    database::set_guild_settings(&ctx.db, guild_id, options.word_type).await?;

    Ok("done!")
}
