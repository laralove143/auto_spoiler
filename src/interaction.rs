use std::mem;

use anyhow::{bail, Result};
use twilight_http::Client;
use twilight_interactions::command::CreateCommand;
use twilight_model::{
    application::interaction::{ApplicationCommand, Interaction, MessageComponentInteraction},
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::ApplicationMarker, Id},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{
    interaction::{
        add_custom_word::CustomWord, add_default_word::AddDefaultWord, tag::Tag, tw::Tw,
    },
    Context,
};

mod add_custom_word;
mod add_default_word;
mod allow;
mod tag;
mod tw;

#[allow(clippy::wildcard_enum_match_arm)]
pub async fn handle(ctx: Context, interaction: Interaction) -> Result<()> {
    match interaction {
        Interaction::ApplicationCommand(cmd) => handle_command(&ctx, *cmd).await,
        Interaction::MessageComponent(component) => handle_component(&ctx, *component).await,
        _ => bail!("unknown interaction: {interaction:#?}"),
    }
}

async fn handle_command(ctx: &Context, mut command: ApplicationCommand) -> Result<()> {
    let command_id = command.id;
    let token = mem::take(&mut command.token);

    let reply = match command.data.name.as_str() {
        "tw" => tw::run(ctx, command).await?,
        "tag" => tag::run(ctx, command).await?,
        "add_custom_word" => add_custom_word::run(ctx, command).await?,
        "add_default_word" => add_default_word::run(ctx, command.data).await?,
        _ => bail!("unknown command: {command:#?}"),
    };

    ctx.http
        .interaction(ctx.application_id)
        .create_response(
            command_id,
            &token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .content(reply.to_owned())
                        .flags(MessageFlags::EPHEMERAL)
                        .build(),
                ),
            },
        )
        .exec()
        .await?;

    Ok(())
}

async fn handle_component(ctx: &Context, mut component: MessageComponentInteraction) -> Result<()> {
    let component_id = component.id;
    let token = mem::take(&mut component.token);

    let response = allow::run(ctx, component).await?;

    ctx.http
        .interaction(ctx.application_id)
        .create_response(component_id, &token, &response)
        .exec()
        .await?;

    Ok(())
}

pub async fn create(http: &Client, application_id: Id<ApplicationMarker>) -> Result<()> {
    let client = http.interaction(application_id);

    // client
    //     .set_global_commands(&[
    //         Tw::create_command().into(),
    //         Tag::create_command().into(),
    //         CustomWord::create_command().into(),
    //     ])
    //     .exec()
    //     .await?;

    client
        .set_guild_commands(
            env!("TEST_GUILD_ID").parse()?,
            &[
                AddDefaultWord::create_command().into(),
                Tw::create_command().into(),
                Tag::create_command().into(),
                CustomWord::create_command().into(),
            ],
        )
        .exec()
        .await?;

    Ok(())
}
