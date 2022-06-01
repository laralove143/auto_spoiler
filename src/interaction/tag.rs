use anyhow::{Context as _, IntoResult, Result};
use twilight_interactions::command::{CommandModel, CommandOption, CreateCommand, CreateOption};
use twilight_model::{
    application::interaction::application_command::CommandData,
    guild::{PartialMember, Permissions},
    id::{marker::ChannelMarker, Id},
};
use twilight_webhook::util::{MinimalMember, MinimalWebhook};

use crate::Context;

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
    command_channel_id: Id<ChannelMarker>,
    member: PartialMember,
) -> Result<&'static str> {
    if !ctx
        .cache
        .permissions()
        .in_channel(ctx.user_id, command_channel_id)?
        .contains(Permissions::MANAGE_WEBHOOKS)
    {
        return Ok("i need the manage webhooks permission in this channel for that :(");
    }

    let options = Tag::from_interaction(data.into())?;
    let content = format!("{} {}", options.message, options.tag.value());
    let channel = ctx.cache.channel(command_channel_id).ok()?;
    let (channel_id, thread_id) = if channel.kind.is_thread() {
        (channel.parent_id.ok()?, Some(channel.id))
    } else {
        (channel.id, None)
    };
    let webhook = ctx
        .webhooks
        .get_infallible(&ctx.http, channel_id, "tw or tag sender")
        .await?;

    MinimalWebhook::try_from(&*webhook)?
        .execute_as_member(
            &ctx.http,
            thread_id,
            &MinimalMember::from_partial_member(
                &member,
                Some(channel.guild_id.ok()?),
                member
                    .user
                    .as_ref()
                    .context("member doesn't have user info")?,
            ),
        )?
        .content(&content)?
        .exec()
        .await?;

    Ok("done")
}
