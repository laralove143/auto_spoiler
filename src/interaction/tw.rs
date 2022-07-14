use anyhow::{IntoResult, Result};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{application::interaction::ApplicationCommand, guild::Permissions};

use crate::{channel_pair, has_permissions, webhook, Context};

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

pub async fn run(ctx: &Context, command: ApplicationCommand) -> Result<&'static str> {
    if !has_permissions(ctx, command.channel_id, Permissions::MANAGE_WEBHOOKS)? {
        return Ok("i need `manage webhooks` permission for this");
    }

    let options = Tw::from_interaction(command.data.into())?;

    let (channel_id, thread_id) = channel_pair(&*ctx.cache.channel(command.channel_id).ok()?)?;
    let member = command.member.ok()?;
    webhook(
        ctx,
        &member,
        member.user.as_ref().ok()?,
        command.guild_id.ok()?,
        channel_id,
        thread_id,
        &format!("tw {} ||{}||", options.tw_type, options.message),
        &[],
    )
    .await?;

    Ok("done!")
}
