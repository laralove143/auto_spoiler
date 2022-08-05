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
use sqlx::PgPool;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_error::ErrorHandler;
use twilight_gateway::{Cluster, EventTypeFlags};
use twilight_http::{client::ClientBuilder, Client};
use twilight_model::{
    application::component::Component,
    channel::{message::AllowedMentions, Channel},
    gateway::{event::Event, Intents},
    guild::{PartialMember, Permissions},
    id::{
        marker::{ApplicationMarker, ChannelMarker, GuildMarker, UserMarker},
        Id,
    },
    user::User,
};
use twilight_webhook::{
    cache::WebhooksCache,
    util::{MinimalMember, MinimalWebhook},
};

mod auto_spoiler;
mod database;
mod interaction;

pub type Context = Arc<ContextInner>;

pub struct ContextInner {
    http: Client,
    cache: InMemoryCache,
    db: PgPool,
    webhooks: WebhooksCache,
    application_id: Id<ApplicationMarker>,
    user_id: Id<UserMarker>,
    owner_channel_id: Id<ChannelMarker>,
    error_handler: ErrorHandler,
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

    let owner_channel_id = http
        .create_private_channel(application.owner.ok()?.id)
        .exec()
        .await?
        .model()
        .await?
        .id;

    let mut error_handler = ErrorHandler::new();
    error_handler
        .channel(owner_channel_id)
        .file("spoiler_bot_errors.txt".into());

    interaction::create(&http, application.id).await?;

    let ctx = Arc::new(ContextInner {
        cache: InMemoryCache::builder()
            .resource_types(resource_types)
            .build(),
        db: database::new().await?,
        webhooks: WebhooksCache::new(),
        user_id: http.current_user().exec().await?.model().await?.id,
        application_id: application.id,
        owner_channel_id,
        error_handler,
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
        ctx.error_handler.handle(&ctx.http, err).await;
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

#[allow(clippy::too_many_arguments)]
async fn webhook(
    ctx: &Context,
    member: &PartialMember,
    user: &User,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    thread_id: Option<Id<ChannelMarker>>,
    content: &str,
    components: &[Component],
) -> Result<()> {
    MinimalWebhook::try_from(
        &*ctx
            .webhooks
            .get_infallible(&ctx.http, channel_id, "tw or tag sender")
            .await?,
    )?
    .execute_as_member(
        &ctx.http,
        thread_id,
        &MinimalMember::from_partial_member(member, Some(guild_id), user),
    )?
    .content(content)?
    .components(components)?
    .exec()
    .await?;

    Ok(())
}

fn has_permissions(
    ctx: &Context,
    channel_id: Id<ChannelMarker>,
    permissions: Permissions,
) -> Result<bool> {
    Ok(ctx
        .cache
        .permissions()
        .in_channel(ctx.user_id, channel_id)?
        .contains(permissions))
}

fn channel_pair(channel: &Channel) -> Result<(Id<ChannelMarker>, Option<Id<ChannelMarker>>)> {
    Ok(if channel.kind.is_thread() {
        (channel.parent_id.ok()?, Some(channel.id))
    } else {
        (channel.id, None)
    })
}
