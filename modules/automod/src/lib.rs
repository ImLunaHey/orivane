use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::{AutomodConfig, AutomodRuleConfig, GuildConfig};
use bot_module_api::{BotModule, MessageCreate, ModuleContext, ModuleEventOutcome};

pub struct AutomodModule;

#[async_trait]
impl BotModule for AutomodModule {
    fn name(&self) -> &'static str {
        "automod"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.automod.enabled
    }
    fn handles_message_create(&self) -> bool {
        true
    }

    async fn on_message_create(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &MessageCreate,
    ) -> Result<Option<ModuleEventOutcome>> {
        if event.author_is_bot {
            return Ok(None);
        }
        let settings = &config.modules.automod;
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

        let Some(rule) = matched_rule(context, settings, event).await? else {
            return Ok(None);
        };
        let reason = format!("Orivane AutoMod: {}", rule.id);
        if rule.action != "warn" {
            context
                .discord
                .delete_message(event.channel_id, event.message_id, &reason)
                .await?;
        }
        match rule.action.as_str() {
            "delete" | "warn" => {}
            "timeout" => {
                context
                    .discord
                    .timeout_member(
                        event.guild_id,
                        event.author_id,
                        u64::from(rule.timeout_seconds),
                        &reason,
                    )
                    .await?;
            }
            "ban" => {
                context
                    .discord
                    .ban_member(event.guild_id, event.author_id, 0, &reason)
                    .await?;
            }
            _ => anyhow::bail!("unsupported automod action"),
        }
        if !settings.response.trim().is_empty() {
            let response = settings
                .response
                .replace("{user}", &format!("<@{}>", event.author_id))
                .replace("{rule}", &rule.id);
            context
                .discord
                .send_message(event.channel_id, &response)
                .await?;
        }
        if let Some(channel_id) = settings.log_channel_id.as_deref() {
            context
                .discord
                .send_message(
                    channel_id
                        .parse()
                        .context("automod log channel is invalid")?,
                    &format!(
                        "AutoMod `{}` matched for <@{}> in <#{}>; action `{}`.",
                        rule.id, event.author_id, event.channel_id, rule.action
                    ),
                )
                .await?;
        }
        Ok(Some(ModuleEventOutcome::message(
            format!("automod.{}.{}", rule.id, rule.action),
            event.message_id,
        )))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RuleMatch {
    id: String,
    action: String,
    timeout_seconds: u32,
}

async fn matched_rule(
    context: &ModuleContext,
    settings: &AutomodConfig,
    event: &MessageCreate,
) -> Result<Option<RuleMatch>> {
    if !settings.rules.is_empty() {
        for rule in settings.rules.iter().filter(|rule| rule.enabled) {
            if rule_matches(context, rule, event).await? {
                return Ok(Some(RuleMatch {
                    id: rule.id.clone(),
                    action: rule.action.clone(),
                    timeout_seconds: rule.timeout_seconds,
                }));
            }
        }
        return Ok(None);
    }
    Ok(legacy_matched_rule(settings, event).map(|id| RuleMatch {
        id: id.into(),
        action: settings.action.clone(),
        timeout_seconds: settings.timeout_seconds,
    }))
}

async fn rule_matches(
    context: &ModuleContext,
    rule: &AutomodRuleConfig,
    event: &MessageCreate,
) -> Result<bool> {
    let normalized = event.content.to_lowercase();
    let threshold = usize::from(rule.threshold);
    Ok(match rule.kind.as_str() {
        "blocked_terms" => rule
            .patterns
            .iter()
            .any(|term| normalized.contains(&term.to_lowercase())),
        "mention_spam" => event.mention_count > threshold,
        "invite_links" => contains_invite(&normalized),
        "external_links" => contains_link(&normalized),
        "caps" => uppercase_percentage(&event.content)
            .is_some_and(|percentage| percentage > u32::from(rule.threshold)),
        "lines" => event.content.lines().count() > threshold,
        "duplicate_spam" => {
            let fingerprint = content_fingerprint(&normalized);
            context
                .services
                .automod_rate_count(
                    event.guild_id,
                    event.author_id,
                    &rule.id,
                    Some(&fingerprint),
                    event.message_id,
                    rule.window_seconds,
                )
                .await?
                >= u64::from(rule.threshold)
        }
        "rapid_spam" => {
            context
                .services
                .automod_rate_count(
                    event.guild_id,
                    event.author_id,
                    &rule.id,
                    None,
                    event.message_id,
                    rule.window_seconds,
                )
                .await?
                >= u64::from(rule.threshold)
        }
        "attachments" => {
            event.attachment_filenames.len() > threshold
                || event.attachment_filenames.iter().any(|filename| {
                    let extension = filename
                        .rsplit_once('.')
                        .map(|(_, extension)| extension.to_lowercase());
                    extension.is_some_and(|extension| {
                        rule.patterns.iter().any(|pattern| {
                            pattern
                                .trim_start_matches('.')
                                .eq_ignore_ascii_case(&extension)
                        })
                    })
                })
        }
        "stickers" => event.sticker_count > threshold,
        "emoji_spam" => emoji_count(&event.content) > threshold,
        "zalgo" => combining_percentage(&event.content) > u32::from(rule.threshold),
        "phishing" => looks_like_phishing(&normalized, &rule.patterns),
        _ => false,
    })
}

fn legacy_matched_rule(settings: &AutomodConfig, event: &MessageCreate) -> Option<&'static str> {
    let normalized = event.content.to_lowercase();
    if settings
        .blocked_words
        .iter()
        .any(|term| !term.is_empty() && normalized.contains(&term.to_lowercase()))
    {
        return Some("blocked_term");
    }
    if event.mention_count > usize::from(settings.max_mentions) {
        return Some("mention_spam");
    }
    if settings.block_invites && contains_invite(&normalized) {
        return Some("invite_link");
    }
    if settings.block_links && contains_link(&normalized) {
        return Some("external_link");
    }
    if event.content.lines().count() > usize::from(settings.max_lines) {
        return Some("excessive_lines");
    }
    if uppercase_percentage(&event.content)
        .is_some_and(|percentage| percentage > u32::from(settings.max_caps_percentage))
    {
        return Some("excessive_caps");
    }
    None
}

fn contains_invite(content: &str) -> bool {
    content.contains("discord.gg/") || content.contains("discord.com/invite/")
}

fn contains_link(content: &str) -> bool {
    content.contains("http://") || content.contains("https://")
}

fn uppercase_percentage(content: &str) -> Option<u32> {
    let (uppercase, total) = content
        .chars()
        .filter(|character| character.is_alphabetic())
        .fold((0_u32, 0_u32), |(uppercase, total), character| {
            (uppercase + u32::from(character.is_uppercase()), total + 1)
        });
    (total >= 8).then(|| uppercase.saturating_mul(100) / total)
}

fn content_fingerprint(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn emoji_count(content: &str) -> usize {
    let custom = content.matches("<:").count() + content.matches("<a:").count();
    custom
        + content
            .chars()
            .filter(|character| matches!(*character as u32, 0x1F000..=0x1FAFF | 0x2600..=0x27BF))
            .count()
}

fn combining_percentage(content: &str) -> u32 {
    let total = content.chars().count() as u32;
    if total == 0 {
        return 0;
    }
    let combining = content.chars().filter(|character| matches!(*character as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)).count() as u32;
    combining.saturating_mul(100) / total
}

fn looks_like_phishing(content: &str, blocked_domains: &[String]) -> bool {
    if !contains_link(content) {
        return false;
    }
    blocked_domains
        .iter()
        .any(|domain| content.contains(&domain.to_lowercase()))
        || content.contains("xn--")
        || content.contains("http://discord")
        || content.contains("https://discord-")
        || content.contains("free nitro")
        || content.contains("steam gift")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};

    use super::*;

    #[tokio::test]
    async fn deletes_times_out_responds_and_logs_without_retaining_content() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.automod.enabled = true;
        config.modules.automod.blocked_words = vec!["secretword".to_owned()];
        config.modules.automod.action = "timeout".to_owned();
        config.modules.automod.timeout_seconds = 300;
        config.modules.automod.log_channel_id = Some("99".to_owned());
        let event = MessageCreate {
            guild_id: 10,
            channel_id: 20,
            message_id: 30,
            author_id: 40,
            author_is_bot: false,
            role_ids: vec![],
            mention_count: 0,
            content: "contains SECRETWORD".to_owned(),
            attachment_filenames: vec![],
            sticker_count: 0,
        };
        let outcome = AutomodModule
            .on_message_create(&context, &config, &event)
            .await
            .unwrap();
        assert_eq!(
            outcome,
            Some(ModuleEventOutcome::message(
                "automod.blocked_term.timeout",
                30
            ))
        );
        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::DeleteMessage {
                    channel_id: 20,
                    message_id: 30,
                    reason: "Orivane AutoMod: blocked_term".to_owned(),
                },
                DiscordCall::TimeoutMember {
                    guild_id: 10,
                    user_id: 40,
                    duration_seconds: 300,
                    reason: "Orivane AutoMod: blocked_term".to_owned(),
                },
                DiscordCall::SendMessage {
                    channel_id: 20,
                    content: "<@40>, that message was removed by AutoMod (blocked_term)."
                        .to_owned(),
                },
                DiscordCall::SendMessage {
                    channel_id: 99,
                    content: "AutoMod `blocked_term` matched for <@40> in <#20>; action `timeout`."
                        .to_owned(),
                },
            ]
        );
        assert!(!format!("{:?}", discord.calls()).contains("SECRETWORD"));
    }

    #[tokio::test]
    async fn ignored_channels_have_no_side_effects() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.automod.ignored_channel_ids = vec!["20".to_owned()];
        let event = MessageCreate {
            guild_id: 10,
            channel_id: 20,
            message_id: 30,
            author_id: 40,
            author_is_bot: false,
            role_ids: vec![],
            mention_count: 99,
            content: "spam".to_owned(),
            attachment_filenames: vec![],
            sticker_count: 0,
        };
        assert_eq!(
            AutomodModule
                .on_message_create(&context, &config, &event)
                .await
                .unwrap(),
            None
        );
        assert!(discord.calls().is_empty());
    }

    #[tokio::test]
    async fn advanced_attachment_rule_uses_its_own_action() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.automod.rules = vec![bot_core::AutomodRuleConfig {
            id: "unsafe-files".into(),
            enabled: true,
            kind: "attachments".into(),
            action: "ban".into(),
            timeout_seconds: 300,
            threshold: 10,
            window_seconds: 0,
            patterns: vec!["exe".into(), "scr".into()],
        }];
        let event = MessageCreate {
            guild_id: 10,
            channel_id: 20,
            message_id: 30,
            author_id: 40,
            author_is_bot: false,
            role_ids: vec![],
            mention_count: 0,
            content: "download".into(),
            attachment_filenames: vec!["update.EXE".into()],
            sticker_count: 0,
        };

        let outcome = AutomodModule
            .on_message_create(&context, &config, &event)
            .await
            .unwrap();
        assert_eq!(outcome.unwrap().action, "automod.unsafe-files.ban");
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::BanMember { user_id: 40, .. }))
        );
    }

    #[tokio::test]
    async fn rapid_spam_uses_the_distributed_rate_counter() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.automod.rules = vec![bot_core::AutomodRuleConfig {
            id: "rapid".into(),
            enabled: true,
            kind: "rapid_spam".into(),
            action: "delete".into(),
            timeout_seconds: 300,
            threshold: 3,
            window_seconds: 10,
            patterns: vec![],
        }];
        let mut event = MessageCreate {
            guild_id: 10,
            channel_id: 20,
            message_id: 30,
            author_id: 40,
            author_is_bot: false,
            role_ids: vec![],
            mention_count: 0,
            content: "hello".into(),
            attachment_filenames: vec![],
            sticker_count: 0,
        };

        assert!(
            AutomodModule
                .on_message_create(&context, &config, &event)
                .await
                .unwrap()
                .is_none()
        );
        event.message_id = 31;
        assert!(
            AutomodModule
                .on_message_create(&context, &config, &event)
                .await
                .unwrap()
                .is_none()
        );
        event.message_id = 32;
        assert!(
            AutomodModule
                .on_message_create(&context, &config, &event)
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            services
                .calls()
                .iter()
                .filter(|call| matches!(
                    call,
                    bot_test_support::ServiceCall::AutomodRateCount { .. }
                ))
                .count(),
            3
        );
    }

    #[test]
    fn detects_phishing_emoji_and_zalgo_without_retaining_content() {
        assert!(looks_like_phishing(
            "claim free nitro at https://example.test",
            &[]
        ));
        assert!(looks_like_phishing("https://xn--dscord-q9a.test", &[]));
        assert_eq!(emoji_count("hello 😀 <:party:123>"), 2);
        assert!(combining_percentage("a\u{0301}\u{0301}\u{0301}") > 50);
        assert!(!looks_like_phishing(
            "ordinary conversation",
            &["bad.test".into()]
        ));
    }
}
