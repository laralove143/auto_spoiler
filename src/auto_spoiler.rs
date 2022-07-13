use anyhow::{IntoResult, Result};
use twilight_model::{channel::Message, guild::Permissions};

use crate::{channel_pair, database, webhook, Context};

pub async fn edit(ctx: Context, message: Message) -> Result<()> {
    if message.author.bot {
        return Ok(());
    }

    let guild_id = message.guild_id.ok()?;
    let mut content = message.content.to_lowercase();
    let mut filter_words = database::words(&ctx.db, guild_id).await?;
    filter_words.retain(|word| content.contains(&word.word));
    if filter_words.is_empty() {
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

    for word in filter_words {
        content = content.replace(&word.word, &format!("||{}||", word.word));
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
