use anyhow::{Context as _, IntoResult, Result};
use twilight_model::{channel::Message, guild::Permissions};
use twilight_webhook::util::{MinimalMember, MinimalWebhook};
use unicode_segmentation::UnicodeSegmentation;

use crate::{database, Context};

pub async fn edit(ctx: Context, message: Message) -> Result<()> {
    if message.author.bot
        || !ctx
            .cache
            .permissions()
            .in_channel(ctx.user_id, message.channel_id)?
            .contains(
                Permissions::VIEW_CHANNEL
                    | Permissions::MANAGE_MESSAGES
                    | Permissions::MANAGE_WEBHOOKS,
            )
    {
        return Ok(());
    }

    let channel = ctx.cache.channel(message.channel_id).ok()?;
    let (channel_id, thread_id) = if channel.kind.is_thread() {
        (channel.parent_id.ok()?, Some(channel.id))
    } else {
        (channel.id, None)
    };
    let member = message.member.context("message doesn't have member info")?;
    let guild_id = message.guild_id.context("message is not in a guild")?;
    let filter_words = database::words(&ctx.db, guild_id).await?;

    let mut edited = false;
    let mut content = String::with_capacity(message.content.len());
    for word in message.content.to_lowercase().split_word_bounds() {
        if filter_words.contains(word) {
            edited = true;
            content.push_str("||");
            content.push_str(word);
            content.push_str("||");
        } else {
            content.push_str(word);
        }
    }

    if !edited {
        return Ok(());
    }

    ctx.http
        .delete_message(thread_id.unwrap_or(channel_id), message.id)
        .exec()
        .await?;

    let webhook = ctx
        .webhooks
        .get_infallible(&ctx.http, channel_id, "tw or tag sender")
        .await?;

    MinimalWebhook::try_from(&*webhook)?
        .execute_as_member(
            &ctx.http,
            thread_id,
            &MinimalMember::from_partial_member(&member, Some(guild_id), &message.author),
        )?
        .content(&content)?
        .exec()
        .await?;

    Ok(())
}
