use std::collections::HashSet;

use anyhow::Result;
use futures_util::TryStreamExt;
use sqlx::{query, query_scalar, sqlite::SqliteConnectOptions, SqlitePool};
use twilight_model::id::{marker::GuildMarker, Id};

use crate::interaction::WordType;

pub async fn new() -> Result<SqlitePool> {
    let db =
        SqlitePool::connect_with(SqliteConnectOptions::new().filename("spoiler.sqlite")).await?;

    Ok(db)
}

#[allow(clippy::integer_arithmetic)]
pub async fn words(db: &SqlitePool, guild_id: Id<GuildMarker>) -> Result<HashSet<String>> {
    let id: i64 = guild_id.get().try_into()?;

    Ok(query_scalar!(
        r#"
        WITH options AS (
            SELECT guild_id, swear_words, trigger_words
            FROM guild_settings
            WHERE guild_id = ?
            UNION ALL
            SELECT null, true, true
        ), settings AS (
           SELECT * FROM options
           LIMIT 1
        )
        SELECT word FROM settings
        INNER JOIN custom_words USING (guild_id)
        UNION ALL
        SELECT word FROM settings
        INNER JOIN default_swear_words ON settings.swear_words
        UNION ALL
        SELECT word FROM settings
        INNER JOIN default_trigger_words ON settings.trigger_words"#,
        id,
    )
    .fetch(db)
    .try_collect()
    .await?)
}

#[allow(clippy::integer_arithmetic)]
pub async fn all_words(db: &SqlitePool, guild_id: Id<GuildMarker>) -> Result<HashSet<String>> {
    let id: i64 = guild_id.get().try_into()?;

    Ok(query_scalar!(
        r#"
        SELECT word FROM custom_words WHERE guild_id=?
        UNION ALL
        SELECT word FROM default_swear_words
        UNION ALL
        SELECT word FROM default_trigger_words"#,
        id,
    )
    .fetch(db)
    .try_collect()
    .await?)
}

#[allow(clippy::integer_arithmetic)]
pub async fn add_custom_word(
    db: &SqlitePool,
    guild_id: Id<GuildMarker>,
    word: String,
) -> Result<()> {
    let id: i64 = guild_id.get().try_into()?;

    query!(
        "INSERT INTO custom_words (guild_id, word) VALUES (?, ?)",
        id,
        word
    )
    .execute(db)
    .await?;

    Ok(())
}

#[allow(clippy::integer_arithmetic)]
pub async fn remove_custom_word(
    db: &SqlitePool,
    guild_id: Id<GuildMarker>,
    word: &str,
) -> Result<()> {
    let id: i64 = guild_id.get().try_into()?;

    query!(
        "DELETE FROM custom_words WHERE guild_id=? AND word=?",
        id,
        word
    )
    .execute(db)
    .await?;

    Ok(())
}

#[allow(clippy::integer_arithmetic)]
pub async fn add_default_word(db: &SqlitePool, word_type: WordType, word: &str) -> Result<()> {
    match word_type {
        WordType::Swear => query!("INSERT INTO default_swear_words (word) VALUES (?)", word),
        WordType::Trigger => query!("INSERT INTO default_trigger_words (word) VALUES (?)", word),
    }
    .execute(db)
    .await?;

    Ok(())
}

#[allow(clippy::integer_arithmetic)]
pub async fn set_guild_settings(
    db: &SqlitePool,
    guild_id: Id<GuildMarker>,
    word_type: Option<WordType>,
) -> Result<()> {
    let id: i64 = guild_id.get().try_into()?;

    match word_type {
        Some(kind) => match kind {
            WordType::Swear => query!(
                "INSERT OR REPLACE INTO guild_settings (guild_id, swear_words) VALUES (?, ?)",
                id,
                false
            ),
            WordType::Trigger => query!(
                "INSERT OR REPLACE INTO guild_settings (guild_id, trigger_words) VALUES (?, ?)",
                id,
                false
            ),
        },
        None => query!("DELETE FROM guild_settings WHERE guild_id=?", id),
    }
    .execute(db)
    .await?;

    Ok(())
}
