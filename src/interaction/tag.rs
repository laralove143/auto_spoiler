use anyhow::{Context as _, Result};
use twilight_interactions::command::{CommandModel, CommandOption, CreateCommand, CreateOption};
use twilight_model::{application::interaction::ApplicationCommand, guild::Permissions};

use crate::{channel_pair, has_permissions, webhook, Context};

#[derive(CreateOption, CommandOption)]
pub enum Tone {
    #[option(name = "joking", value = "/j")]
    Joking,
    #[option(name = "sarcastic", value = "/s")]
    Sarcastic,
    #[option(name = "genuine", value = "/gen")]
    Genuine,
    #[option(name = "reference", value = "/ref")]
    Reference,
    #[option(name = "not mad", value = "/nm")]
    NotMad,
    #[option(name = "not at you", value = "/nay")]
    NobodyHere,
    #[option(name = "not being rude", value = "/nbr")]
    NotBeingRude,
    #[option(name = "off topic", value = "/ot")]
    OffTopic,
    #[option(name = "lyrics", value = "/ly")]
    Lyrics,
    #[option(name = "metaphor", value = "/m")]
    Metaphor,
    #[option(name = "rhetorical question", value = "/rh")]
    RhetoricalQuestion,
    #[option(name = "exaggeration", value = "/ex")]
    Exaggeration,
    #[option(name = "romantic", value = "/r")]
    Romantic,
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "tag", desc = "put a tone tag at the end of your message")]
pub struct Tag {
    #[command(name = "message", desc = "your message")]
    message: String,
    #[command(name = "tag", desc = "the tone tag")]
    tag: Tone,
}

pub async fn run(ctx: &Context, command: ApplicationCommand) -> Result<&'static str> {
    if !has_permissions(ctx, command.channel_id, Permissions::MANAGE_WEBHOOKS)? {
        return Ok("i need `manage webhooks` permissions for this");
    }

    let options = Tag::from_interaction(command.data.into())?;

    let (channel_id, thread_id) = channel_pair(
        &*ctx
            .cache
            .channel(command.channel_id)
            .context("channel is not cached")?,
    )?;
    let member = command.member.context("command doesn't have a member")?;
    webhook(
        ctx,
        &member,
        member
            .user
            .as_ref()
            .context("command member doesn't have a user")?,
        command
            .guild_id
            .context("command doesn't have a guild id")?,
        channel_id,
        thread_id,
        &format!("{} {}", options.message, options.tag.value()),
        &[],
    )
    .await?;

    Ok("done!")
}
