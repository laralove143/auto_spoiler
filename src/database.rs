use anyhow::Result;
use sqlx::{query_file, query_file_as, PgPool};
use twilight_model::id::{marker::GuildMarker, Id};

pub struct Word {
    id: i32,
    guild_id: Option<i64>,
    pub word: String,
}

pub async fn new() -> Result<PgPool> {
    Ok(PgPool::connect("postgres://spoiler").await?)
}

#[allow(clippy::integer_arithmetic, clippy::panic)]
pub async fn words(db: &PgPool, guild_id: Id<GuildMarker>) -> Result<Vec<Word>> {
    Ok(
        query_file_as!(Word, "src/database/query_words.sql", encode(guild_id))
            .fetch_all(db)
            .await?,
    )
}

#[allow(clippy::integer_arithmetic, clippy::panic)]
pub async fn add_custom_word(db: &PgPool, guild_id: Id<GuildMarker>, word: String) -> Result<()> {
    query_file!(
        "src/database/insert_custom_word.sql",
        encode(guild_id),
        word
    )
    .execute(db)
    .await?;

    Ok(())
}

#[allow(clippy::integer_arithmetic, clippy::panic)]
pub async fn add_default_word(db: &PgPool, word: String) -> Result<()> {
    query_file!("src/database/insert_default_word.sql", word)
        .execute(db)
        .await?;

    Ok(())
}

#[allow(clippy::cast_possible_wrap, clippy::as_conversions)]
const fn encode<T>(id: Id<T>) -> i64 {
    id.get() as i64
}
