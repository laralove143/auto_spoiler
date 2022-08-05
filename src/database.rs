use anyhow::Result;
use sqlx::{query, query_as, PgPool};
use twilight_model::id::{marker::GuildMarker, Id};

pub struct Word {
    pub id: i32,
    pub word: String,
}

pub async fn new() -> Result<PgPool> {
    let db = PgPool::connect("postgres://localhost/spoiler_bot").await?;

    query!(
        r#"
        CREATE TABLE IF NOT EXISTS words (
            id serial PRIMARY KEY,
            guild_id bigint,
            word text NOT NULL
        );
        "#
    )
    .execute(&db)
    .await?;

    query!(
        r#"
        CREATE INDEX IF NOT EXISTS words_guild_id_index ON words (guild_id);
        "#
    )
    .execute(&db)
    .await?;

    query!(
        r#"
        CREATE TABLE IF NOT EXISTS allowed_words (
            guild_id bigint NOT NULL PRIMARY KEY,
            word_id int NOT NULL
        );
        "#
    )
    .execute(&db)
    .await?;

    Ok(db)
}

#[allow(clippy::integer_arithmetic, clippy::panic)]
pub async fn words(db: &PgPool, guild_id: Id<GuildMarker>) -> Result<Vec<Word>> {
    Ok(query_as!(
        Word,
        r#"
        SELECT
            id AS "id!",
            word AS "word!"
        FROM
            words
        WHERE
            guild_id = $1
            OR guild_id IS NULL
            AND NOT EXISTS (
                SELECT
                    1
                FROM
                    allowed_words
                WHERE
                    guild_id = $1
                    AND word_id = id);
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

pub async fn delete_word(db: &PgPool, id: i32) -> Result<()> {
    #[allow(clippy::panic, clippy::integer_arithmetic)]
    query!(
        r#"
        DELETE FROM words
        WHERE id = $1
        "#,
        id
    )
    .execute(db)
    .await?;

    Ok(())
}

#[allow(clippy::cast_possible_wrap, clippy::as_conversions)]
const fn encode<T>(id: Id<T>) -> i64 {
    id.get() as i64
}
