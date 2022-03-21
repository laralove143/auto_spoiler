use anyhow::{Context as _, Result};
use twilight_interactions::command::{CommandModel, CommandOption, CreateCommand, CreateOption};
use twilight_model::{
    application::interaction::application_command::CommandData,
    guild::{PartialMember, Permissions},
    id::{marker::ChannelMarker, Id},
};

use crate::{webhooks, Context};

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

pub async fn run(
    ctx: &Context,
    data: CommandData,
    channel_id: Id<ChannelMarker>,
    member: PartialMember,
) -> Result<&'static str> {
    if !ctx
        .cache
        .permissions()
        .in_channel(ctx.user_id, channel_id)?
        .contains(Permissions::MANAGE_WEBHOOKS)
    {
        return Ok("i need the manage webhooks permission in this channel for that :(");
    }

    let options = Tag::from_interaction(data.into())?;
    let content = format!("{} {}", options.message, options.tag.value());

    webhooks::send_as_member(
        ctx,
        channel_id,
        &member,
        member
            .user
            .as_ref()
            .context("member doesn't have user info")?,
        &content,
    )
    .await?;

    Ok("done")
}
