use anyhow::{Context as _, IntoResult, Result};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    guild::{PartialMember, Permissions},
    id::{marker::ChannelMarker, Id},
};
use twilight_webhook::util::{MinimalMember, MinimalWebhook};

use crate::Context;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "tw",
    desc = "warn users that the message may be triggering, putting it in spoilers"
)]
pub struct Tw {
    #[command(name = "message", desc = "your possibly triggering message")]
    message: String,
    #[command(name = "tw_type", desc = "why your message may be triggering")]
    tw_type: String,
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

    let options = Tw::from_interaction(data.into())?;
    let content = format!("tw {} ||{}||", options.tw_type, options.message);
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
