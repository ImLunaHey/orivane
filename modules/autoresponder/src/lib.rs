use anyhow::Result;
use async_trait::async_trait;
use bot_core::{AutoresponseConfig, GuildConfig};
use bot_discord::RichEmbed;
use bot_module_api::{BotModule, MessageCreate, ModuleContext, ModuleEventOutcome};

pub struct AutoresponderModule;

#[async_trait]
impl BotModule for AutoresponderModule {
    fn name(&self) -> &'static str {
        "autoresponder"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.autoresponder.enabled
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
        let Some((rule_index, rule)) = config
            .modules
            .autoresponder
            .responses
            .iter()
            .enumerate()
            .find(|(_, rule)| matches_rule(rule, event))
        else {
            return Ok(None);
        };
        if !context
            .services
            .claim_module_cooldown(
                "autoresponder",
                &rule_index.to_string(),
                event.guild_id,
                event.author_id,
                rule.cooldown_seconds,
            )
            .await?
        {
            return Ok(None);
        }
        let response = rule
            .response
            .replace("{user}", &format!("<@{}>", event.author_id))
            .replace("{channel}", &format!("<#{}>", event.channel_id))
            .replace("{server_id}", &event.guild_id.to_string());
        if rule.embed_title.trim().is_empty() && rule.embed_color == 0 {
            context
                .discord
                .send_message(event.channel_id, &response)
                .await?;
        } else {
            context
                .discord
                .send_rich_embed(
                    event.channel_id,
                    &RichEmbed {
                        title: rule.embed_title.clone(),
                        description: response,
                        color: rule.embed_color,
                        timestamp: true,
                        ..RichEmbed::default()
                    },
                )
                .await?;
        }
        for reaction in rule
            .reactions
            .iter()
            .map(|reaction| reaction.trim())
            .filter(|reaction| !reaction.is_empty())
            .take(5)
        {
            context
                .discord
                .add_reaction(event.channel_id, event.message_id, reaction)
                .await?;
        }
        Ok(Some(ModuleEventOutcome::message(
            "autoresponder.responded",
            event.message_id,
        )))
    }
}

fn matches_rule(rule: &AutoresponseConfig, event: &MessageCreate) -> bool {
    if !rule.enabled
        || (!rule.allowed_channel_ids.is_empty()
            && !rule
                .allowed_channel_ids
                .iter()
                .any(|id| id == &event.channel_id.to_string()))
        || rule.ignored_role_ids.iter().any(|id| {
            id.parse::<u64>()
                .is_ok_and(|id| event.role_ids.contains(&id))
        })
    {
        return false;
    }
    let (content, trigger) = if rule.case_sensitive {
        (event.content.clone(), rule.trigger.clone())
    } else {
        (event.content.to_lowercase(), rule.trigger.to_lowercase())
    };
    match rule.match_mode.as_str() {
        "exact" => content == trigger,
        "starts_with" => content.starts_with(&trigger),
        "regex" => regex::RegexBuilder::new(&rule.trigger)
            .case_insensitive(!rule.case_sensitive)
            .size_limit(256 * 1024)
            .build()
            .is_ok_and(|pattern| pattern.is_match(&event.content)),
        _ => content.contains(&trigger),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};
    use std::sync::Arc;

    #[tokio::test]
    async fn responds_to_an_allowed_case_insensitive_trigger() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.autoresponder.enabled = true;
        config.modules.autoresponder.responses = vec![AutoresponseConfig {
            enabled: true,
            trigger: "hello".into(),
            response: "Hi {user} in {channel}".into(),
            match_mode: "contains".into(),
            case_sensitive: false,
            allowed_channel_ids: vec!["20".into()],
            ignored_role_ids: vec![],
            cooldown_seconds: 30,
            reactions: vec!["👋".into()],
            embed_title: String::new(),
            embed_color: 0,
        }];
        let outcome = AutoresponderModule
            .on_message_create(
                &context,
                &config,
                &MessageCreate {
                    guild_id: 10,
                    channel_id: 20,
                    message_id: 30,
                    author_id: 40,
                    author_is_bot: false,
                    role_ids: vec![],
                    mention_count: 0,
                    content: "HELLO there".into(),
                    attachment_filenames: vec![],
                    sticker_count: 0,
                },
            )
            .await
            .unwrap();
        assert!(outcome.is_some());
        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::SendMessage {
                    channel_id: 20,
                    content: "Hi <@40> in <#20>".into()
                },
                DiscordCall::AddReaction {
                    channel_id: 20,
                    message_id: 30,
                    emoji: "👋".into(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn supports_regex_and_rich_embed_responses() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.autoresponder.enabled = true;
        config.modules.autoresponder.responses = vec![AutoresponseConfig {
            enabled: true,
            trigger: r"^help\s+me$".into(),
            response: "Hello {user}".into(),
            match_mode: "regex".into(),
            case_sensitive: false,
            allowed_channel_ids: vec![],
            ignored_role_ids: vec![],
            cooldown_seconds: 60,
            reactions: vec![],
            embed_title: "Support".into(),
            embed_color: 0x5865f2,
        }];
        AutoresponderModule
            .on_message_create(
                &context,
                &config,
                &MessageCreate {
                    guild_id: 10,
                    channel_id: 20,
                    message_id: 30,
                    author_id: 40,
                    author_is_bot: false,
                    role_ids: vec![],
                    mention_count: 0,
                    content: "HELP me".into(),
                    attachment_filenames: vec![],
                    sticker_count: 0,
                },
            )
            .await
            .unwrap();
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::SendRichEmbed { embed, .. }
                if embed.title == "Support" && embed.description == "Hello <@40>"
        )));
    }
}
