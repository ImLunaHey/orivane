use std::{env, sync::Arc, time::Instant};

use anyhow::{Context, Result};
use bot_core::{Infrastructure, NewAuditEvent};
use bot_discord::{CommandDefinition, CommandInteraction, DiscordApi, TwilightDiscord};
use bot_module_api::{AuditNotice, BotModule, MemberJoin, ModuleContext};
use bot_module_automod::AutomodModule;
use bot_module_autorole::AutoroleModule;
use bot_module_custom_commands::CustomCommandsModule;
use bot_module_leveling::LevelingModule;
use bot_module_logging::LoggingModule;
use bot_module_moderation::ModerationModule;
use bot_module_starboard::StarboardModule;
use bot_module_tickets::TicketsModule;
use bot_module_utility::UtilityModule;
use bot_module_welcome::WelcomeModule;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_model::application::interaction::InteractionData;
use twilight_model::gateway::payload::incoming::GuildCreate;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let token = required_env("DISCORD_TOKEN")?;
    let database_url = required_env("DATABASE_URL")?;
    let redis_url = required_env("REDIS_URL")?;

    let discord: Arc<dyn DiscordApi> = Arc::new(TwilightDiscord::connect(token.clone()).await?);
    register_commands(discord.as_ref()).await?;
    let context = ModuleContext { discord };
    let infrastructure = Infrastructure::connect(&database_url, &redis_url).await?;
    let modules: Vec<Arc<dyn BotModule>> = vec![
        Arc::new(UtilityModule),
        Arc::new(WelcomeModule),
        Arc::new(AutoroleModule),
        Arc::new(LoggingModule),
        Arc::new(ModerationModule),
        Arc::new(AutomodModule),
        Arc::new(LevelingModule),
        Arc::new(StarboardModule),
        Arc::new(TicketsModule),
        Arc::new(CustomCommandsModule),
    ];
    let intents = Intents::GUILDS | Intents::GUILD_MEMBERS;
    let mut shard = Shard::new(ShardId::ONE, token, intents);

    tracing::info!("gateway worker starting");
    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let event = match item {
            Ok(event) => event,
            Err(error) => {
                tracing::warn!(%error, "failed to receive gateway event");
                continue;
            }
        };

        match event {
            Event::GuildCreate(guild) => {
                if let GuildCreate::Available(guild) = *guild {
                    let guild_id = guild.id.to_string();
                    if let Err(error) = infrastructure
                        .upsert_connected_guild(
                            &guild_id,
                            &guild.name,
                            guild.member_count.unwrap_or(guild.members.len() as u64),
                            shard.id().number().into(),
                            &guild.owner_id.to_string(),
                        )
                        .await
                    {
                        tracing::error!(%guild_id, %error, "failed to update connected guild registry");
                    }
                }
            }
            Event::GuildDelete(guild) => {
                let guild_id = guild.id.to_string();
                if guild.unavailable != Some(true)
                    && let Err(error) = infrastructure.mark_guild_disconnected(&guild_id).await
                {
                    tracing::error!(%guild_id, %error, "failed to mark guild disconnected");
                }
            }
            Event::MemberAdd(member) => {
                let guild_id = member.guild_id.to_string();
                let username = member.user.name.clone();
                let display_name = member
                    .nick
                    .clone()
                    .or_else(|| member.user.global_name.clone())
                    .unwrap_or_else(|| username.clone());
                let config = match infrastructure.guild_config(&guild_id).await {
                    Ok(config) => config,
                    Err(error) => {
                        tracing::error!(%guild_id, %error, "could not load guild configuration for member join");
                        continue;
                    }
                };
                let event = MemberJoin {
                    guild_id: member.guild_id.get(),
                    guild_name: infrastructure
                        .connected_guild_name(&guild_id)
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "the server".to_owned()),
                    user_id: member.user.id.get(),
                    username,
                    display_name,
                };
                for module in modules
                    .iter()
                    .filter(|module| module.handles_member_join() && module.enabled(&config))
                {
                    let module = Arc::clone(module);
                    let context = context.clone();
                    let config = config.clone();
                    let event = event.clone();
                    let infrastructure = infrastructure.clone();
                    let guild_id = guild_id.clone();
                    tokio::spawn(async move {
                        let result = module.on_member_join(&context, &config, &event).await;
                        let mut audit = NewAuditEvent::guild(
                            &guild_id,
                            "system",
                            format!("module.{}.member_join", module.name()),
                        );
                        audit.actor_kind = "system".to_owned();
                        audit.source = "discord_gateway".to_owned();
                        audit.target_type = "member".to_owned();
                        audit.target_id = Some(event.user_id.to_string());
                        audit.metadata = serde_json::json!({ "module": module.name() });
                        if let Err(error) = result {
                            audit.outcome = "failure".to_owned();
                            audit.reason = Some(error.to_string());
                        }
                        if let Err(error) = infrastructure.record_audit_event(&audit).await {
                            tracing::error!(%error, "failed to persist member join module audit event");
                        }
                    });
                }
            }
            Event::InteractionCreate(interaction) => {
                let Some(InteractionData::ApplicationCommand(command)) = interaction.data.as_ref()
                else {
                    continue;
                };
                let Some(guild_id) = interaction.guild_id else {
                    continue;
                };
                let config = match infrastructure.guild_config(&guild_id.to_string()).await {
                    Ok(config) => config,
                    Err(error) => {
                        tracing::error!(%guild_id, %error, "could not load guild configuration");
                        continue;
                    }
                };

                for module in &modules {
                    if !module
                        .commands()
                        .iter()
                        .any(|definition| definition.name == command.name)
                    {
                        continue;
                    }
                    let actor_id = interaction
                        .author_id()
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "unknown-user".to_owned());
                    let command = CommandInteraction {
                        id: interaction.id.get(),
                        token: interaction.token.clone(),
                        guild_id: guild_id.get(),
                        command_name: command.name.clone(),
                    };
                    let started_at = Instant::now();
                    let module_enabled = module.enabled(&config);
                    let result = if module_enabled {
                        module.on_command(&context, &config, &command).await
                    } else {
                        context
                            .discord
                            .respond_to_command(
                                &command,
                                bot_discord::CommandResponse::ephemeral(format!(
                                    "The {} module is disabled in this server.",
                                    module.name()
                                )),
                            )
                            .await
                            .map_err(anyhow::Error::from)
                    };
                    let latency_ms =
                        started_at.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
                    let mut audit = NewAuditEvent::guild(
                        guild_id.to_string(),
                        actor_id,
                        format!("slash_command.{}", command.command_name),
                    );
                    audit.source = "discord_gateway".to_owned();
                    audit.target_type = "slash_command".to_owned();
                    audit.target_id = Some(command.command_name.clone());
                    audit.metadata = serde_json::json!({ "module": module.name(), "interaction_id": command.id.to_string(), "latency_ms": latency_ms });
                    if !module_enabled {
                        audit.outcome = "denied".to_owned();
                        audit.reason = Some("module is disabled for this server".to_owned());
                    }
                    if let Err(error) = &result {
                        audit.outcome = "failure".to_owned();
                        audit.reason = Some(error.to_string());
                        tracing::error!(module = module.name(), %error, "module failed");
                    }
                    if let Err(error) = infrastructure.record_audit_event(&audit).await {
                        tracing::error!(%error, "failed to persist slash command audit event");
                    }
                    let notice = AuditNotice {
                        guild_id: guild_id.get(),
                        action: audit.action.clone(),
                        actor_id: audit.actor_id.clone(),
                        outcome: audit.outcome.clone(),
                    };
                    for logging_module in modules.iter().filter(|candidate| {
                        candidate.handles_audit_notice() && candidate.enabled(&config)
                    }) {
                        if let Err(error) = logging_module
                            .on_audit_notice(&context, &config, &notice)
                            .await
                        {
                            tracing::error!(module = logging_module.name(), %error, "failed to deliver server log notice");
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

async fn register_commands(discord: &dyn DiscordApi) -> Result<()> {
    let commands: Vec<CommandDefinition> = bot_module_utility::COMMANDS.to_vec();
    discord.register_commands(&commands).await?;
    tracing::info!("registered global application commands");
    Ok(())
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn,twilight_gateway=warn".into()),
        )
        .init();
}
