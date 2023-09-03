use anyhow::{Context as _, Result};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{application::interaction::ApplicationCommand, guild::Permissions};

use crate::{database, Context};

#[derive(CommandModel, CreateCommand)]
#[command(name = "add_custom_word", desc = "add your own word to censor")]
pub struct AddCustomWord {
    #[command(name = "word", desc = "the word to add")]
    word: String,
    #[command(
        name = "suggest",
        desc = "set true to tell my developer to add this word to the default list"
    )]
    suggest: bool,
}

pub async fn run(ctx: &Context, command: ApplicationCommand) -> Result<&'static str> {
    let member = command.member.context("command doesn't have a member")?;
    if !member
        .permissions
        .context("member doesn't have permissions attached")?
        .contains(Permissions::MANAGE_GUILD)
    {
        return Ok("you need the manage guild permission to use this");
    }
    let guild_id = command
        .guild_id
        .context("command doesn't have a guild id")?;

    let options = AddCustomWord::from_interaction(command.data.into())?;
    let word = options.word.to_lowercase();

    if database::words(&ctx.db, guild_id)
        .await?
        .iter()
        .any(|w| w.word == word)
    {
        return Ok("this word is already added!");
    }

    if options.suggest {
        let user = member
            .user
            .context("command member doesn't have a user attached")?;

        ctx.http
            .create_message(ctx.owner_channel_id)
            .content(&format!(
                "{}#{} suggested a word: {}",
                user.name, user.discriminator, word
            ))?
            .exec()
            .await?;
    }

    database::add_custom_word(&ctx.db, guild_id, word).await?;

    Ok("done!")
}
