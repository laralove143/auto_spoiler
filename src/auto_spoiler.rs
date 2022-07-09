use anyhow::{IntoResult, Result};
use twilight_model::{channel::Message, guild::Permissions};
use unicode_segmentation::UnicodeSegmentation;

use crate::{channel_pair, database, webhook, Context};

pub async fn edit(ctx: Context, message: Message) -> Result<()> {
    if message.author.bot {
        return Ok(());
    }

    let guild_id = message.guild_id.ok()?;

    let filter_words = database::words(&ctx.db, guild_id).await?;
    let mut content = String::with_capacity(message.content.len());
    let mut edited = false;
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

    let permissions = ctx
        .cache
        .permissions()
        .in_channel(ctx.user_id, message.channel_id)?;
    if !permissions.contains(Permissions::MANAGE_MESSAGES | Permissions::MANAGE_WEBHOOKS) {
        if permissions.contains(Permissions::SEND_MESSAGES) {
            ctx.http
                .create_message(message.channel_id)
                .content(
                    "there's a word to put in spoilers here but i need `manage messages` and \
                     `manage webhooks` permissions first",
                )?
                .exec()
                .await?;
        }
        return Ok(());
    }

    let channel = ctx.cache.channel(message.channel_id).ok()?;
    let (channel_id, thread_id) = channel_pair(&channel)?;
    let member = message.member.ok()?;
    webhook(
        &ctx,
        &member,
        &message.author,
        message.guild_id.ok()?,
        channel_id,
        thread_id,
        &content,
    )
    .await?;

    ctx.http
        .delete_message(thread_id.unwrap_or(channel_id), message.id)
        .exec()
        .await?;

    Ok(())
}
