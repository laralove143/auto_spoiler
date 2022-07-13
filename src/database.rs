use anyhow::Result;
use sqlx::{query, query_as, PgPool};
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
    Ok(query_as!(
        Word,
        r#"
        SELECT
            id AS "id!",
            guild_id,
            word AS "word!"
        FROM
            words
        WHERE
            guild_id = $1
        UNION ALL
        SELECT
            id,
            guild_id,
            word
        FROM
            words
        WHERE
            guild_id IS NULL
            AND id NOT IN (
                SELECT
                    word_id
                FROM
                    allowed_words
                WHERE
                    guild_id = $1);
        "#,
        encode(guild_id)
    )
    .fetch_all(db)
    .await?)
}

#[allow(clippy::integer_arithmetic, clippy::panic)]
pub async fn add_custom_word(db: &PgPool, guild_id: Id<GuildMarker>, word: String) -> Result<()> {
    query!(
        r#"
        INSERT INTO words (guild_id, word)
            VALUES ($1, $2)
        "#,
        encode(guild_id),
        word
    )
    .execute(db)
    .await?;

    Ok(())
}

#[allow(clippy::integer_arithmetic, clippy::panic)]
pub async fn add_default_word(db: &PgPool, word: String) -> Result<()> {
    query!(
        r#"
        INSERT INTO words (word)
            VALUES ($1)
        "#,
        word
    )
    .execute(db)
    .await?;

    Ok(())
}

#[allow(clippy::cast_possible_wrap, clippy::as_conversions)]
const fn encode<T>(id: Id<T>) -> i64 {
    id.get() as i64
}
