use anyhow::{Context as _, Result};
use twilight_model::{
    application::interaction::MessageComponentInteraction,
    channel::message::MessageFlags,
    guild::Permissions,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{database, Context};

pub async fn run(
    ctx: &Context,
    mut component: MessageComponentInteraction,
) -> Result<InteractionResponse> {
    if !component
        .member
        .context("component interaction doesn't have a member")?
        .permissions
        .context("component interaction member doesn't have permissions attached")?
        .contains(Permissions::MANAGE_GUILD)
    {
        return Ok(InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .content("you need `manage server` permission for this".to_owned())
                    .flags(MessageFlags::EPHEMERAL)
                    .build(),
            ),
        });
    }

    database::delete_word(&ctx.db, component.data.custom_id.parse()?).await?;
    component.message.content.retain(|c| c != '|');

    Ok(InteractionResponse {
        kind: InteractionResponseType::UpdateMessage,
        data: Some(
            InteractionResponseDataBuilder::new()
                .content(component.message.content)
                .components([])
                .flags(MessageFlags::EPHEMERAL)
                .build(),
        ),
    })
}
