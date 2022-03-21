use anyhow::Result;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::application_command::CommandData;

use crate::{database, interaction::WordType, Context};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "add_default_word",
    desc = "add a word to censor",
    default_permission = false
)]
pub struct AddDefaultWord {
    #[command(name = "word", desc = "the new word")]
    word: String,
    #[command(name = "word_type", desc = "the type of the word")]
    word_type: WordType,
}

pub async fn run(ctx: &Context, data: CommandData) -> Result<&'static str> {
    let options = AddDefaultWord::from_interaction(data.into())?;

    database::add_default_word(&ctx.db, options.word_type, &options.word).await?;

    Ok("done!")
}
