use anyhow::{bail, Context as _, Result};
use twilight_http::Client;
use twilight_interactions::command::{CommandOption, CreateCommand, CreateOption};
use twilight_model::{
    application::{
        command::permissions::{CommandPermissions, CommandPermissionsType},
        interaction::Interaction,
    },
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{
        marker::{ApplicationMarker, UserMarker},
        Id,
    },
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{
    interaction::{
        add_default_word::AddDefaultWord, allow::Allow, custom_word::CustomWord, tag::Tag, tw::Tw,
    },
    Context, GUILD_ID,
};

mod add_default_word;
mod allow;
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

    let command = match interaction {
        Interaction::ApplicationCommand(cmd) => *cmd,
        _ => bail!("unknown interaction: {interaction:#?}"),
    };

    let reply = match command.data.name.as_str() {
        "tw" => {
            tw::run(
                &ctx,
                command.data,
                command.channel_id,
                command.member.context("the command is sent in a dm")?,
            )
            .await?
        }
        "tag" => {
            tag::run(
                &ctx,
                command.data,
                command.channel_id,
                command.member.context("the command is sent in a dm")?,
            )
            .await?
        }
        "allow" => {
            allow::run(
                &ctx,
                command.data,
                command.guild_id.context("the command is sent in a dm")?,
                command.member.context("the command is sent in a dm")?,
            )
            .await?
        }
        "custom_word" => {
            custom_word::run(
                &ctx,
                command.data,
                command.guild_id.context("the command is sent in a dm")?,
                command.member.context("the command is sent in a dm")?,
            )
            .await?
        }
        "add_default_word" => add_default_word::run(&ctx, command.data).await?,
        _ => bail!("unknown command: {command:#?}"),
    };

    client
        .create_response(
            command.id,
            &command.token,
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

pub async fn create(
    http: &Client,
    application_id: Id<ApplicationMarker>,
    owner_id: Id<UserMarker>,
) -> Result<()> {
    let client = http.interaction(application_id);

    let mut permissions = Vec::new();
    for command in client
        .set_global_commands(&[
            Tw::create_command().into(),
            Tag::create_command().into(),
            Allow::create_command().into(),
            CustomWord::create_command().into(),
            AddDefaultWord::create_command().into(),
        ])
        .exec()
        .await?
        .model()
        .await?
    {
        if !command.default_permission.unwrap_or(true) {
            permissions.push((
                command
                    .id
                    .context("command doesn't have id attached to it")?,
                CommandPermissions {
                    id: CommandPermissionsType::User(owner_id),
                    permission: true,
                },
            ));
        }
    }

    client
        .set_command_permissions(GUILD_ID, &permissions)?
        .exec()
        .await?;

    Ok(())
}
