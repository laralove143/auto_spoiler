#![warn(clippy::cargo, clippy::nursery, clippy::pedantic, clippy::restriction)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_same,
    clippy::pattern_type_mismatch
)]

use std::{env, sync::Arc};

use anyhow::{IntoResult, Result};
use futures_util::StreamExt;
use sqlx::SqlitePool;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Cluster, EventTypeFlags};
use twilight_http::{client::ClientBuilder, Client};
use twilight_model::{
    channel::message::AllowedMentions,
    gateway::{event::Event, Intents},
    id::{
        marker::{ApplicationMarker, ChannelMarker, GuildMarker, UserMarker},
        Id,
    },
};
use twilight_webhook::cache::Cache as WebhooksCache;

const GUILD_ID: Id<GuildMarker> = Id::new(903_367_565_349_384_202);

mod auto_spoiler;
mod database;
mod interaction;

pub type Context = Arc<ContextInner>;

pub struct ContextInner {
    http: Client,
    cache: InMemoryCache,
    db: SqlitePool,
    webhooks: WebhooksCache,
    application_id: Id<ApplicationMarker>,
    user_id: Id<UserMarker>,
    owner_channel_id: Id<ChannelMarker>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let intents = Intents::GUILDS
        | Intents::GUILD_WEBHOOKS
        | Intents::GUILD_MESSAGES
        | Intents::MESSAGE_CONTENT;
    let event_types = EventTypeFlags::INTERACTION_CREATE
        | EventTypeFlags::WEBHOOKS_UPDATE
        | EventTypeFlags::MESSAGE_CREATE
        | EventTypeFlags::GUILD_CREATE
        | EventTypeFlags::GUILD_UPDATE
        | EventTypeFlags::GUILD_DELETE
        | EventTypeFlags::CHANNEL_CREATE
        | EventTypeFlags::CHANNEL_UPDATE
        | EventTypeFlags::CHANNEL_DELETE
        | EventTypeFlags::THREAD_CREATE
        | EventTypeFlags::THREAD_DELETE
        | EventTypeFlags::THREAD_UPDATE
        | EventTypeFlags::THREAD_LIST_SYNC
        | EventTypeFlags::THREAD_MEMBER_UPDATE
        | EventTypeFlags::THREAD_MEMBERS_UPDATE
        | EventTypeFlags::ROLE_CREATE
        | EventTypeFlags::ROLE_UPDATE
        | EventTypeFlags::ROLE_DELETE
        | EventTypeFlags::MEMBER_ADD
        | EventTypeFlags::MEMBER_UPDATE
        | EventTypeFlags::MEMBER_REMOVE;
    let resource_types =
        ResourceType::GUILD | ResourceType::CHANNEL | ResourceType::MEMBER | ResourceType::ROLE;

    let token = env::var("SPOILER_BOT_TOKEN")?;

    let (cluster, mut events) = Cluster::builder(token.clone(), intents)
        .event_types(event_types)
        .build()
        .await?;
    let cluster_spawn = Arc::new(cluster);
    tokio::spawn(async move { cluster_spawn.up().await });

    let http = ClientBuilder::new()
        .token(token)
        .default_allowed_mentions(AllowedMentions::default())
        .build();

    let application = http
        .current_user_application()
        .exec()
        .await?
        .model()
        .await?;

    interaction::create(&http, application.id).await?;

    let ctx = Arc::new(ContextInner {
        cache: InMemoryCache::builder()
            .resource_types(resource_types)
            .build(),
        db: database::new().await?,
        webhooks: WebhooksCache::new(),
        user_id: http.current_user().exec().await?.model().await?.id,
        owner_channel_id: http
            .create_private_channel(application.owner.ok()?.id)
            .exec()
            .await?
            .model()
            .await?
            .id,
        application_id: application.id,
        http,
    });

    while let Some((_, event)) = events.next().await {
        ctx.cache.update(&event);
        tokio::spawn(handle_event(Arc::clone(&ctx), event));
    }

    Ok(())
}

#[allow(clippy::print_stderr, clippy::wildcard_enum_match_arm)]
async fn handle_event(ctx: Context, event: Event) {
    ctx.cache.update(&event);
    ctx.webhooks.update(&event);
    if let Err(err) = _handle_event(Arc::clone(&ctx), event).await {
        eprintln!("{err}");
        if let Err(e) = inform_owner(&ctx).await {
            eprintln!("informing the owner also failed: {e}");
        }
    }
}

#[allow(clippy::print_stderr, clippy::wildcard_enum_match_arm)]
async fn _handle_event(ctx: Context, event: Event) -> Result<()> {
    match event {
        Event::InteractionCreate(interaction) => interaction::handle(ctx, interaction.0).await?,
        Event::WebhooksUpdate(update) => {
            ctx.webhooks
                .validate(
                    &ctx.http,
                    update.channel_id,
                    ctx.cache
                        .permissions()
                        .in_channel(ctx.user_id, update.channel_id)?,
                )
                .await?;
        }
        Event::MessageCreate(message) => auto_spoiler::edit(ctx, (*message).0).await?,
        _ => (),
    }
    Ok(())
}

async fn inform_owner(ctx: &Context) -> Result<()> {
    ctx.http
        .create_message(ctx.owner_channel_id)
        .content("there was an error, i printed it")?
        .exec()
        .await?;

    Ok(())
}
