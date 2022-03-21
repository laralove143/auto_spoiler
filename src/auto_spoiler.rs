use anyhow::{Context as _, Result};
use twilight_model::{channel::Message, guild::Permissions};
use unicode_segmentation::UnicodeSegmentation;

use crate::{database, webhooks, Context};

pub async fn edit(ctx: Context, message: Message) -> Result<()> {
    if message.author.bot
        || !ctx
            .cache
            .permissions()
            .in_channel(ctx.user_id, message.channel_id)?
            .contains(Permissions::MANAGE_MESSAGES | Permissions::MANAGE_WEBHOOKS)
    {
        return Ok(());
    }

    let member = message.member.context("message doesn't have member info")?;
    let filter_words = database::words(
        &ctx.db,
        message.guild_id.context("message is not in a guild")?,
    )
    .await?;

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
        .delete_message(message.channel_id, message.id)
        .exec()
        .await?;

    webhooks::send_as_member(&ctx, message.channel_id, &member, &message.author, &content).await?;

    Ok(())
}
