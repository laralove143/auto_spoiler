use std::mem;

use anyhow::{bail, Result};
use twilight_http::Client;
use twilight_interactions::command::{CommandOption, CreateCommand, CreateOption};
use twilight_model::{
    application::interaction::Interaction,
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::ApplicationMarker, Id},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{
    interaction::{add_default_word::AddDefaultWord, custom_word::CustomWord, tag::Tag, tw::Tw},
    Context,
};

mod add_default_word;
mod custom_word;
mod tag;
mod tw;

#[derive(CreateOption, CommandOption)]
pub enum WordType {
    #[option(name = "swear-words", value = "swear-words")]
    Swear,
    #[option(name = "trigger-words", value = "trigger-words")]
    Trigger,
}

#[allow(clippy::wildcard_enum_match_arm)]
pub async fn handle(ctx: Context, interaction: Interaction) -> Result<()> {
    let client = ctx.http.interaction(ctx.application_id);

    let mut command = match interaction {
        Interaction::ApplicationCommand(cmd) => *cmd,
        _ => bail!("unknown interaction: {interaction:#?}"),
    };
    let command_id = command.id;
    let token = mem::take(&mut command.token);

    let reply = match command.data.name.as_str() {
        "tw" => tw::run(&ctx, command).await?,
        "tag" => tag::run(&ctx, command).await?,
        "custom_word" => custom_word::run(&ctx, command).await?,
        "add_default_word" => add_default_word::run(&ctx, command.data).await?,
        _ => bail!("unknown command: {command:#?}"),
    };

    client
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
