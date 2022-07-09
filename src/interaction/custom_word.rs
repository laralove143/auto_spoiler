use anyhow::{IntoResult, Result};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{application::interaction::ApplicationCommand, guild::Permissions};

use crate::{database, Context};

#[derive(CommandModel, CreateCommand)]
#[command(name = "add", desc = "add a custom word")]
pub struct Add {
    #[command(name = "word", desc = "the word to add")]
    word: String,
    #[command(
        name = "suggest",
        desc = "set true to tell my developer to add this word to the default list"
    )]
    suggest: bool,
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "remove", desc = "remove a custom word you've added before")]
pub struct Remove {
    #[command(name = "word", desc = "the word to remove")]
    pub word: String,
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "custom_word",
    desc = "add your own word to censor or remove a custom word"
)]
pub enum CustomWord {
    #[command(name = "add")]
    Add(Add),
    #[command(name = "remove")]
    Remove(Remove),
}

pub async fn run(ctx: &Context, command: ApplicationCommand) -> Result<&'static str> {
    let member = command.member.ok()?;
    if !member.permissions.ok()?.contains(Permissions::MANAGE_GUILD) {
        return Ok("you need the manage guild permission to use this");
    }
    let guild_id = command.guild_id.ok()?;

    let option = CustomWord::from_interaction(command.data.into())?;

    match option {
        CustomWord::Add(add) => {
            if database::all_words(&ctx.db, guild_id)
                .await?
                .contains(&add.word)
            {
                return Ok("this word is already added!");
            }

            if add.suggest {
                let user = member.user.ok()?;

                ctx.http
                    .create_message(ctx.owner_channel_id)
                    .content(&format!(
                        "{}#{} suggested a word: {}",
                        user.name, user.discriminator, add.word
                    ))?
                    .exec()
                    .await?;
            }

            database::add_custom_word(&ctx.db, guild_id, add.word).await?;
        }
        CustomWord::Remove(remove) => {
            if !database::all_words(&ctx.db, guild_id)
                .await?
                .contains(&remove.word)
            {
                return Ok("the word isn't there anyway..");
            }
            database::remove_custom_word(&ctx.db, guild_id, &remove.word).await?;
        }
    }

    Ok("done!")
}
