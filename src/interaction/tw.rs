use anyhow::{Context as _, Result};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    guild::{PartialMember, Permissions},
    id::{marker::ChannelMarker, Id},
};

use crate::{webhooks, Context};

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

    let options = Tw::from_interaction(data.into())?;
    let content = format!("tw {} ||{}||", options.tw_type, options.message);

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
