use std::collections::HashSet;

use anyhow::{IntoResult, Result};
use futures_util::{StreamExt, TryStreamExt};
use sqlx::{query, query_scalar, PgPool};
use twilight_model::id::{marker::GuildMarker, Id};

pub async fn new() -> Result<PgPool> {
    Ok(PgPool::connect("postgres://spoiler").await?)
}

#[allow(clippy::integer_arithmetic, clippy::panic)]
pub async fn words(db: &PgPool, guild_id: Id<GuildMarker>) -> Result<HashSet<String>> {
    query_scalar!(
        "SELECT word
        FROM custom_words
        WHERE guild_id = $1
        UNION ALL
        SELECT word
        FROM default_words
        WHERE id NOT IN (SELECT word_id FROM allowed_words WHERE guild_id = $1);",
        encode(guild_id),
    )
    .fetch(db)
    .map(|row| match row {
        Ok(Some(word)) => Ok(word),
        Ok(None) => None.ok(),
        Err(e) => Err(e.into()),
    })
    .try_collect()
    .await
}

#[allow(clippy::integer_arithmetic, clippy::panic)]
pub async fn add_custom_word(db: &PgPool, guild_id: Id<GuildMarker>, word: String) -> Result<()> {
    query!(
        "INSERT INTO custom_words (guild_id, word) VALUES ($1, $2)",
        encode(guild_id),
        word
    )
    .execute(db)
    .await?;

    Ok(())
}

#[allow(clippy::integer_arithmetic, clippy::panic)]
pub async fn add_default_word(db: &PgPool, word: String) -> Result<()> {
    query!("INSERT INTO default_words (word) VALUES ($1)", word)
        .execute(db)
        .await?;

    Ok(())
}

#[allow(clippy::cast_possible_wrap, clippy::as_conversions)]
const fn encode<T>(id: Id<T>) -> i64 {
    id.get() as i64
}
