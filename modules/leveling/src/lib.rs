use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::{GuildConfig, LevelingConfig};
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse, RichEmbed, RichEmbedField,
};
use bot_module_api::{
    BotModule, MessageCreate, ModuleContext, ModuleEventOutcome, VoiceStateChange,
    XpAdjustmentInput, XpAwardInput, XpProfile,
};

pub struct LevelingModule;

const MANAGE_GUILD: u64 = 1 << 5;
const XP_OPTIONS: &[CommandOptionDefinition] = &[
    CommandOptionDefinition {
        name: "member",
        description: "Member whose XP should change",
        kind: CommandOptionKind::User,
        required: true,
        min_value: None,
        max_value: None,
        min_length: None,
        max_length: None,
    },
    CommandOptionDefinition {
        name: "amount",
        description: "XP amount",
        kind: CommandOptionKind::Integer,
        required: true,
        min_value: Some(0),
        max_value: Some(10_000_000),
        min_length: None,
        max_length: None,
    },
];

pub const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        name: "rank",
        description: "Show a member's XP, level, and server rank",
        guild_only: true,
        default_permissions: None,
        options: &[CommandOptionDefinition {
            name: "member",
            description: "Member to inspect; defaults to you",
            kind: CommandOptionKind::User,
            required: false,
            min_value: None,
            max_value: None,
            min_length: None,
            max_length: None,
        }],
    },
    CommandDefinition {
        name: "leaderboard",
        description: "Show the server's highest-ranked members",
        guild_only: true,
        default_permissions: None,
        options: &[],
    },
    CommandDefinition {
        name: "xp-add",
        description: "Add XP to a member",
        guild_only: true,
        default_permissions: Some(MANAGE_GUILD),
        options: XP_OPTIONS,
    },
    CommandDefinition {
        name: "xp-remove",
        description: "Remove XP from a member",
        guild_only: true,
        default_permissions: Some(MANAGE_GUILD),
        options: XP_OPTIONS,
    },
    CommandDefinition {
        name: "xp-set",
        description: "Set a member's XP total",
        guild_only: true,
        default_permissions: Some(MANAGE_GUILD),
        options: XP_OPTIONS,
    },
];

#[async_trait]
impl BotModule for LevelingModule {
    fn name(&self) -> &'static str {
        "leveling"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.leveling.enabled
    }
    fn handles_message_create(&self) -> bool {
        true
    }
    fn handles_voice_state_change(&self) -> bool {
        true
    }

    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }

    async fn on_command(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &CommandInteraction,
    ) -> Result<()> {
        let response = match interaction.command_name.as_str() {
            "rank" => {
                let user_id = interaction.user("member").unwrap_or(interaction.actor_id);
                let profile = context
                    .services
                    .xp_profile(interaction.guild_id, user_id)
                    .await?;
                CommandResponse::public_embed(rank_card(&profile, &config.modules.leveling))
            }
            "leaderboard" => {
                let profiles = context
                    .services
                    .xp_leaderboard(interaction.guild_id, 10)
                    .await?;
                if profiles.is_empty() {
                    CommandResponse::public("No members have earned XP yet.")
                } else {
                    let rows = profiles
                        .iter()
                        .map(|profile| {
                            format!(
                                "#{} <@{}> · level {} · {} XP",
                                profile.rank, profile.user_id, profile.level, profile.total_xp
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    CommandResponse::public(format!("**Server leaderboard**\n{rows}"))
                }
            }
            "xp-add" | "xp-remove" | "xp-set" => {
                if interaction.actor_permissions & MANAGE_GUILD == 0 {
                    bail!("Manage Server is required to administer XP");
                }
                let user_id = interaction.user("member").context("member is required")?;
                let amount = interaction
                    .integer("amount")
                    .context("amount is required")?;
                let (operation, amount) = match interaction.command_name.as_str() {
                    "xp-add" => ("add", amount),
                    "xp-remove" => ("add", -amount),
                    _ => ("set", amount),
                };
                let profile = context
                    .services
                    .adjust_xp(XpAdjustmentInput {
                        guild_id: interaction.guild_id,
                        user_id,
                        amount,
                        operation: operation.into(),
                    })
                    .await?;
                apply_role_rewards(
                    context,
                    interaction.guild_id,
                    user_id,
                    &[],
                    &config.modules.leveling,
                    profile.level,
                )
                .await?;
                CommandResponse::ephemeral(format!(
                    "<@{user_id}> now has {} XP at level {}.",
                    profile.total_xp, profile.level
                ))
            }
            _ => return Ok(()),
        };
        context
            .discord
            .respond_to_command(interaction, response)
            .await?;
        Ok(())
    }

    async fn on_message_create(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &MessageCreate,
    ) -> Result<Option<ModuleEventOutcome>> {
        if event.author_is_bot
            || event.content.trim().chars().count()
                < usize::from(config.modules.leveling.minimum_message_length)
        {
            return Ok(None);
        }
        let settings = &config.modules.leveling;
        if settings
            .ignored_channel_ids
            .iter()
            .any(|id| id == &event.channel_id.to_string())
            || event.role_ids.iter().any(|role_id| {
                settings
                    .ignored_role_ids
                    .iter()
                    .any(|id| id == &role_id.to_string())
            })
        {
            return Ok(None);
        }
        if settings.duplicate_message_window_seconds > 0 {
            let fingerprint = message_fingerprint(&event.content);
            let count = context
                .services
                .automod_rate_count(
                    event.guild_id,
                    event.author_id,
                    "leveling-antifarm",
                    Some(&fingerprint),
                    event.message_id,
                    settings.duplicate_message_window_seconds,
                )
                .await?;
            if count > 1 {
                return Ok(None);
            }
        }
        let range = settings
            .xp_per_message_max
            .saturating_sub(settings.xp_per_message_min)
            .saturating_add(1);
        let amount = settings.xp_per_message_min
            + u16::try_from(event.message_id % u64::from(range)).unwrap_or_default();
        let result = context
            .services
            .award_xp(XpAwardInput {
                guild_id: event.guild_id,
                user_id: event.author_id,
                amount,
                cooldown_seconds: settings.cooldown_seconds,
            })
            .await?;
        if !result.awarded {
            return Ok(None);
        }
        if result.leveled_up && settings.announce_level_up {
            let channel_id = settings
                .announcement_channel_id
                .as_deref()
                .map(str::parse)
                .transpose()
                .context("level announcement channel is invalid")?
                .unwrap_or(event.channel_id);
            let message = settings
                .announcement_message
                .replace("{user}", &format!("<@{}>", event.author_id))
                .replace("{level}", &result.level.to_string())
                .replace("{xp}", &result.total_xp.to_string());
            context.discord.send_message(channel_id, &message).await?;
        }
        if result.leveled_up {
            apply_role_rewards(
                context,
                event.guild_id,
                event.author_id,
                &event.role_ids,
                settings,
                result.level,
            )
            .await?;
        }
        Ok(Some(ModuleEventOutcome::message(
            "leveling.xp_awarded",
            event.message_id,
        )))
    }

    async fn on_voice_state_change(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &VoiceStateChange,
    ) -> Result<Option<ModuleEventOutcome>> {
        let settings = &config.modules.leveling;
        let Some(previous_channel_id) = event.previous_channel_id else {
            return Ok(None);
        };
        if event.user_is_bot
            || !event.previous_session_eligible
            || !settings.voice_xp_enabled
            || settings
                .ignored_voice_channel_ids
                .iter()
                .any(|id| id == &previous_channel_id.to_string())
        {
            return Ok(None);
        }
        let minutes = (event.previous_session_seconds / 60)
            .min(u64::from(settings.voice_xp_max_session_minutes));
        if minutes == 0 {
            return Ok(None);
        }
        let amount = minutes
            .saturating_mul(u64::from(settings.voice_xp_per_minute))
            .min(i64::MAX as u64) as i64;
        let profile = context
            .services
            .adjust_xp(XpAdjustmentInput {
                guild_id: event.guild_id,
                user_id: event.user_id,
                amount,
                operation: "add".into(),
            })
            .await?;
        apply_role_rewards(
            context,
            event.guild_id,
            event.user_id,
            &[],
            settings,
            profile.level,
        )
        .await?;
        Ok(Some(ModuleEventOutcome {
            action: "leveling.voice_xp_awarded".into(),
            target_type: "member".into(),
            target_id: Some(event.user_id.to_string()),
        }))
    }
}

fn rank_card(profile: &XpProfile, settings: &LevelingConfig) -> RichEmbed {
    let current_floor = 100_u64.saturating_mul(u64::from(profile.level).pow(2));
    let next = 100_u64.saturating_mul(u64::from(profile.level.saturating_add(1)).pow(2));
    let progress = profile.total_xp.saturating_sub(current_floor);
    let needed = next.saturating_sub(current_floor).max(1);
    let filled = usize::try_from(progress.saturating_mul(10) / needed)
        .unwrap_or(10)
        .min(10);
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(10 - filled));
    RichEmbed {
        title: format!(
            "Level {} · Rank {}",
            profile.level,
            if profile.rank == 0 {
                "unranked".into()
            } else {
                format!("#{}", profile.rank)
            }
        ),
        description: format!(
            "<@{}>\n`{bar}` {progress} / {needed} XP to next level",
            profile.user_id
        ),
        color: settings.rank_card_color,
        footer_text: settings.rank_card_footer.clone(),
        fields: vec![
            RichEmbedField {
                name: "Total XP".into(),
                value: profile.total_xp.to_string(),
                inline: true,
            },
            RichEmbedField {
                name: "Eligible messages".into(),
                value: profile.message_count.to_string(),
                inline: true,
            },
        ],
        ..RichEmbed::default()
    }
}

fn message_fingerprint(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

async fn apply_role_rewards(
    context: &ModuleContext,
    guild_id: u64,
    user_id: u64,
    current_roles: &[u64],
    settings: &LevelingConfig,
    level: u32,
) -> Result<()> {
    let mut earned = settings
        .role_rewards
        .iter()
        .filter(|reward| reward.level <= level)
        .collect::<Vec<_>>();
    earned.sort_by_key(|reward| reward.level);
    let keep = earned.last().map(|reward| reward.role_id.as_str());
    for reward in &earned {
        let role_id = reward
            .role_id
            .parse()
            .context("level reward role is invalid")?;
        if (!settings.remove_previous_rewards || keep == Some(reward.role_id.as_str()))
            && !current_roles.contains(&role_id)
        {
            context
                .discord
                .add_member_role(guild_id, user_id, role_id)
                .await?;
        }
    }
    if settings.remove_previous_rewards {
        for reward in settings
            .role_rewards
            .iter()
            .filter(|reward| Some(reward.role_id.as_str()) != keep)
        {
            let role_id = reward
                .role_id
                .parse()
                .context("level reward role is invalid")?;
            if current_roles.is_empty() || current_roles.contains(&role_id) {
                context
                    .discord
                    .remove_member_role(guild_id, user_id, role_id, "Level reward progression")
                    .await?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};

    use super::*;

    #[tokio::test]
    async fn awards_deterministic_xp_through_the_service_boundary() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.leveling.enabled = true;
        config.modules.leveling.xp_per_message_min = 10;
        config.modules.leveling.xp_per_message_max = 20;
        let event = MessageCreate {
            guild_id: 10,
            channel_id: 20,
            message_id: 33,
            author_id: 40,
            author_is_bot: false,
            role_ids: vec![],
            mention_count: 0,
            content: "A useful message".to_owned(),
            attachment_filenames: vec![],
            sticker_count: 0,
        };
        assert!(
            LevelingModule
                .on_message_create(&context, &config, &event)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            matches!(&services.calls()[0], ServiceCall::AutomodRateCount { rule_id, .. } if rule_id == "leveling-antifarm")
        );
        assert_eq!(
            services.calls()[1],
            ServiceCall::AwardXp(XpAwardInput {
                guild_id: 10,
                user_id: 40,
                amount: 10,
                cooldown_seconds: 60,
            })
        );
        assert!(discord.calls().is_empty());
    }

    #[tokio::test]
    async fn rank_reads_the_persisted_profile() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let command = CommandInteraction {
            id: 1,
            token: "token".to_owned(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 40,
            actor_role_ids: vec![],
            actor_permissions: u64::MAX,
            command_name: "rank".to_owned(),
            options: Default::default(),
        };
        LevelingModule
            .on_command(&context, &GuildConfig::for_guild("10"), &command)
            .await
            .unwrap();
        assert_eq!(
            services.calls(),
            vec![ServiceCall::XpProfile {
                guild_id: 10,
                user_id: 40,
            }]
        );
        let DiscordCall::RespondToCommand { response, .. } = &discord.calls()[0] else {
            panic!("expected command response")
        };
        let embed = response
            .embed
            .as_ref()
            .expect("rank response should be an embed");
        assert_eq!(embed.title, "Level 2 · Rank #3");
        assert!(embed.description.contains("225 / 500 XP"));
        assert_eq!(embed.color, 0x5865f2);
    }

    #[tokio::test]
    async fn awards_voice_xp_and_grants_the_highest_reward() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.leveling.voice_xp_enabled = true;
        config.modules.leveling.voice_xp_per_minute = 50;
        config.modules.leveling.role_rewards = vec![bot_core::LevelRewardConfig {
            level: 1,
            role_id: "70".into(),
        }];
        let event = VoiceStateChange {
            guild_id: 10,
            user_id: 40,
            previous_channel_id: Some(20),
            channel_id: None,
            previous_session_seconds: 600,
            previous_session_eligible: true,
            user_is_bot: false,
            username: "luna".into(),
        };

        let outcome = LevelingModule
            .on_voice_state_change(&context, &config, &event)
            .await
            .unwrap();
        assert_eq!(outcome.unwrap().action, "leveling.voice_xp_awarded");
        assert!(
            matches!(&services.calls()[0], ServiceCall::AdjustXp(input) if input.amount == 500)
        );
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::AddMemberRole { role_id: 70, .. }))
        );
    }

    #[tokio::test]
    async fn xp_administration_requires_manage_server() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord),
            services: Arc::new(services.clone()),
        };
        let mut command = CommandInteraction {
            id: 1,
            token: "token".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 40,
            actor_role_ids: vec![],
            actor_permissions: 0,
            command_name: "xp-add".into(),
            options: std::collections::BTreeMap::from([
                ("member".into(), bot_discord::CommandValue::User(50)),
                ("amount".into(), bot_discord::CommandValue::Integer(100)),
            ]),
        };
        assert!(
            LevelingModule
                .on_command(&context, &GuildConfig::for_guild("10"), &command)
                .await
                .is_err()
        );
        command.actor_permissions = MANAGE_GUILD;
        LevelingModule
            .on_command(&context, &GuildConfig::for_guild("10"), &command)
            .await
            .unwrap();
        assert!(
            matches!(&services.calls()[0], ServiceCall::AdjustXp(input) if input.user_id == 50 && input.amount == 100)
        );
    }
}
