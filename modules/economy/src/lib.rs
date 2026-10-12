use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::{EconomyConfig, GuildConfig};
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse,
};
use bot_module_api::{BotModule, ModuleContext};
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct EconomyModule;
const MANAGE_GUILD: u64 = 1 << 5;
const MEMBER_OPTION: CommandOptionDefinition = CommandOptionDefinition {
    name: "member",
    description: "Member to inspect or pay",
    kind: CommandOptionKind::User,
    required: true,
    min_value: None,
    max_value: None,
    min_length: None,
    max_length: None,
};
const OPTIONAL_MEMBER: &[CommandOptionDefinition] = &[CommandOptionDefinition {
    required: false,
    ..MEMBER_OPTION
}];
const AMOUNT_OPTION: CommandOptionDefinition = CommandOptionDefinition {
    name: "amount",
    description: "Currency amount",
    kind: CommandOptionKind::Integer,
    required: true,
    min_value: Some(1),
    max_value: Some(1_000_000_000),
    min_length: None,
    max_length: None,
};
const PAY_OPTIONS: &[CommandOptionDefinition] = &[MEMBER_OPTION, AMOUNT_OPTION];
const WAGER_OPTION: CommandOptionDefinition = CommandOptionDefinition {
    name: "wager",
    description: "Amount to wager",
    kind: CommandOptionKind::Integer,
    required: true,
    min_value: Some(1),
    max_value: Some(1_000_000_000),
    min_length: None,
    max_length: None,
};
const WAGER_OPTIONS: &[CommandOptionDefinition] = &[WAGER_OPTION];
const COINFLIP_OPTIONS: &[CommandOptionDefinition] = &[
    WAGER_OPTION,
    CommandOptionDefinition {
        name: "choice",
        description: "heads or tails",
        kind: CommandOptionKind::String,
        required: true,
        min_value: None,
        max_value: None,
        min_length: Some(4),
        max_length: Some(5),
    },
];
const BUY_OPTIONS: &[CommandOptionDefinition] = &[CommandOptionDefinition {
    name: "item",
    description: "Shop item ID",
    kind: CommandOptionKind::String,
    required: true,
    min_value: None,
    max_value: None,
    min_length: Some(1),
    max_length: Some(32),
}];
const fn command(
    name: &'static str,
    description: &'static str,
    permission: Option<u64>,
    options: &'static [CommandOptionDefinition],
) -> CommandDefinition {
    CommandDefinition {
        name,
        description,
        guild_only: true,
        default_permissions: permission,
        options,
    }
}
pub const COMMANDS: &[CommandDefinition] = &[
    command(
        "balance",
        "Show a member's economy balance",
        None,
        OPTIONAL_MEMBER,
    ),
    command("daily", "Claim the daily currency reward", None, &[]),
    command("work", "Earn currency after a cooldown", None, &[]),
    command(
        "pay",
        "Transfer currency to another member",
        None,
        PAY_OPTIONS,
    ),
    command(
        "economy-leaderboard",
        "Show the richest server members",
        None,
        &[],
    ),
    command("shop", "List server shop items", None, &[]),
    command("buy", "Purchase a server shop item", None, BUY_OPTIONS),
    command("coinflip", "Wager on a coin flip", None, COINFLIP_OPTIONS),
    command("slots", "Play the slot machine", None, WAGER_OPTIONS),
    command(
        "coins-add",
        "Add currency to a member",
        Some(MANAGE_GUILD),
        PAY_OPTIONS,
    ),
    command(
        "coins-remove",
        "Remove currency from a member",
        Some(MANAGE_GUILD),
        PAY_OPTIONS,
    ),
    command(
        "coins-set",
        "Set a member's currency balance",
        Some(MANAGE_GUILD),
        PAY_OPTIONS,
    ),
];

#[async_trait]
impl BotModule for EconomyModule {
    fn name(&self) -> &'static str {
        "economy"
    }
    fn enabled(&self, c: &GuildConfig) -> bool {
        c.modules.economy.enabled
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
        let settings = &config.modules.economy;
        let response = match interaction.command_name.as_str() {
            "balance" => {
                let user = interaction.user("member").unwrap_or(interaction.actor_id);
                let profile = context
                    .services
                    .economy_profile(interaction.guild_id, user)
                    .await?;
                CommandResponse::public(format!(
                    "<@{user}> has **{}** {} {}.",
                    profile.balance, settings.currency_symbol, settings.currency_name
                ))
            }
            "daily" => {
                claim(
                    context,
                    interaction,
                    settings,
                    "daily",
                    settings.daily_amount,
                    86_400,
                )
                .await?
            }
            "work" => {
                let amount =
                    deterministic_range(interaction.id, settings.work_min, settings.work_max);
                claim(
                    context,
                    interaction,
                    settings,
                    "work",
                    amount,
                    u64::from(settings.work_cooldown_seconds),
                )
                .await?
            }
            "pay" => {
                let member = interaction.user("member").context("member is required")?;
                if member == interaction.actor_id {
                    bail!("you cannot pay yourself")
                };
                let amount = amount(interaction)?;
                let (_, recipient, tax) = context
                    .services
                    .economy_transfer(
                        interaction.guild_id,
                        interaction.actor_id,
                        member,
                        amount,
                        settings.transfer_tax_percent,
                        &format!("interaction:{}", interaction.id),
                    )
                    .await?;
                CommandResponse::public(format!(
                    "Transferred **{}** {} to <@{}>{}. Their balance is **{}**.",
                    amount.saturating_sub(tax),
                    settings.currency_symbol,
                    member,
                    if tax > 0 {
                        format!(" after a {tax} {} tax", settings.currency_symbol)
                    } else {
                        String::new()
                    },
                    recipient.balance
                ))
            }
            "economy-leaderboard" => {
                let profiles = context
                    .services
                    .economy_leaderboard(interaction.guild_id, 10)
                    .await?;
                let rows = profiles
                    .iter()
                    .enumerate()
                    .map(|(index, p)| {
                        format!(
                            "#{} <@{}> · {} {}",
                            index + 1,
                            p.user_id,
                            p.balance,
                            settings.currency_symbol
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                CommandResponse::public(if rows.is_empty() {
                    "No economy activity yet.".into()
                } else {
                    format!("**Economy leaderboard**\n{rows}")
                })
            }
            "shop" => {
                let rows = settings
                    .shop_items
                    .iter()
                    .map(|item| {
                        format!(
                            "`{}` **{}** · {} {}\n{}",
                            item.id,
                            item.name,
                            item.price,
                            settings.currency_symbol,
                            item.description
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                CommandResponse::public(if rows.is_empty() {
                    "The server shop is empty.".into()
                } else {
                    format!("**Server shop**\n{rows}")
                })
            }
            "buy" => {
                let item_id = interaction.string("item").context("item is required")?;
                let item = settings
                    .shop_items
                    .iter()
                    .find(|item| item.id == item_id)
                    .context("shop item was not found")?;
                let reference = format!("interaction:{}:purchase", interaction.id);
                context
                    .services
                    .economy_change_balance(
                        interaction.guild_id,
                        interaction.actor_id,
                        -i64::try_from(item.price).context("item price is invalid")?,
                        "shop_purchase",
                        &reference,
                    )
                    .await?;
                if let Some(role_id) = item.role_id.as_deref() {
                    let role_id = role_id.parse().context("shop role is invalid")?;
                    if let Err(error) = context
                        .discord
                        .add_member_role(interaction.guild_id, interaction.actor_id, role_id)
                        .await
                    {
                        let _ = context
                            .services
                            .economy_change_balance(
                                interaction.guild_id,
                                interaction.actor_id,
                                i64::try_from(item.price).unwrap_or(i64::MAX),
                                "shop_refund",
                                &format!("{reference}:refund"),
                            )
                            .await;
                        return Err(error.into());
                    }
                }
                CommandResponse::ephemeral(format!(
                    "Purchased **{}** for {} {}.",
                    item.name, item.price, settings.currency_symbol
                ))
            }
            "coinflip" => {
                ensure_games(settings)?;
                let wager = wager(interaction, settings)?;
                let choice = interaction
                    .string("choice")
                    .context("choice is required")?
                    .to_lowercase();
                if !matches!(choice.as_str(), "heads" | "tails") {
                    bail!("choice must be heads or tails")
                };
                let result = if seeded(interaction) % 2 == 0 {
                    "heads"
                } else {
                    "tails"
                };
                let won = choice == result;
                settle_game(
                    context,
                    interaction,
                    wager,
                    won.then_some(wager)
                        .map_or(-i64::try_from(wager).unwrap_or(i64::MAX), |profit| {
                            i64::try_from(profit).unwrap_or(i64::MAX)
                        }),
                    if won { "coinflip_win" } else { "coinflip_loss" },
                )
                .await?;
                CommandResponse::public(format!(
                    "The coin landed **{result}**. You {} **{wager}** {}.",
                    if won { "won" } else { "lost" },
                    settings.currency_symbol
                ))
            }
            "slots" => {
                ensure_games(settings)?;
                let wager = wager(interaction, settings)?;
                let seed = seeded(interaction);
                let symbols = ["🍒", "🍋", "⭐", "💎"];
                let reels = [
                    symbols[(seed % 4) as usize],
                    symbols[((seed / 7) % 4) as usize],
                    symbols[((seed / 31) % 4) as usize],
                ];
                let multiplier = if reels[0] == reels[1] && reels[1] == reels[2] {
                    5
                } else if reels[0] == reels[1] || reels[1] == reels[2] || reels[0] == reels[2] {
                    2
                } else {
                    0
                };
                let delta = if multiplier == 0 {
                    -i64::try_from(wager).unwrap_or(i64::MAX)
                } else {
                    i64::try_from(wager.saturating_mul(multiplier - 1)).unwrap_or(i64::MAX)
                };
                settle_game(
                    context,
                    interaction,
                    wager,
                    delta,
                    if multiplier == 0 {
                        "slots_loss"
                    } else {
                        "slots_win"
                    },
                )
                .await?;
                CommandResponse::public(format!(
                    "{} {} {} · {}",
                    reels[0],
                    reels[1],
                    reels[2],
                    if multiplier == 0 {
                        format!("lost {wager} {}", settings.currency_symbol)
                    } else {
                        format!("won {} {}", wager * multiplier, settings.currency_symbol)
                    }
                ))
            }
            "coins-add" | "coins-remove" | "coins-set" => {
                if interaction.actor_permissions & MANAGE_GUILD == 0 {
                    bail!("Manage Server is required")
                };
                let user = interaction.user("member").context("member is required")?;
                let requested = i64::try_from(amount(interaction)?).context("amount is invalid")?;
                let current = context
                    .services
                    .economy_profile(interaction.guild_id, user)
                    .await?;
                let delta = match interaction.command_name.as_str() {
                    "coins-add" => requested,
                    "coins-remove" => -requested,
                    _ => requested - i64::try_from(current.balance).unwrap_or(i64::MAX),
                };
                let profile = context
                    .services
                    .economy_change_balance(
                        interaction.guild_id,
                        user,
                        delta,
                        interaction.command_name.as_str(),
                        &format!("interaction:{}", interaction.id),
                    )
                    .await?;
                CommandResponse::ephemeral(format!(
                    "<@{user}> now has {} {}.",
                    profile.balance, settings.currency_symbol
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
}

fn amount(i: &CommandInteraction) -> Result<u64> {
    u64::try_from(i.integer("amount").context("amount is required")?).context("amount is invalid")
}
fn wager(i: &CommandInteraction, s: &EconomyConfig) -> Result<u64> {
    let wager = u64::try_from(i.integer("wager").context("wager is required")?)
        .context("wager is invalid")?;
    if !(s.minimum_wager..=s.maximum_wager).contains(&wager) {
        bail!(
            "wager must be between {} and {}",
            s.minimum_wager,
            s.maximum_wager
        )
    }
    Ok(wager)
}
fn ensure_games(s: &EconomyConfig) -> Result<()> {
    if !s.games_enabled {
        bail!("economy games are disabled")
    };
    Ok(())
}
fn seeded(i: &CommandInteraction) -> u64 {
    let mut h = DefaultHasher::new();
    (i.id, i.actor_id, i.guild_id).hash(&mut h);
    h.finish()
}
fn deterministic_range(seed: u64, min: u64, max: u64) -> u64 {
    min + seed % (max - min + 1)
}
async fn claim(
    c: &ModuleContext,
    i: &CommandInteraction,
    s: &EconomyConfig,
    kind: &str,
    amount: u64,
    cooldown: u64,
) -> Result<CommandResponse> {
    let result = c
        .services
        .economy_claim(
            i.guild_id,
            i.actor_id,
            kind,
            amount,
            cooldown,
            &format!("interaction:{}:{kind}", i.id),
        )
        .await?;
    Ok(match result.profile {
        Some(profile) => CommandResponse::public(format!(
            "You earned **{amount}** {}. Balance: **{}**.",
            s.currency_symbol, profile.balance
        )),
        None => CommandResponse::ephemeral(format!(
            "Try again in {}.",
            duration(result.retry_after_seconds)
        )),
    })
}
async fn settle_game(
    c: &ModuleContext,
    i: &CommandInteraction,
    _: u64,
    delta: i64,
    kind: &str,
) -> Result<()> {
    c.services
        .economy_change_balance(
            i.guild_id,
            i.actor_id,
            delta,
            kind,
            &format!("interaction:{}:{kind}", i.id),
        )
        .await?;
    Ok(())
}
fn duration(seconds: u64) -> String {
    if seconds >= 3600 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else if seconds >= 60 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_discord::CommandValue;
    use bot_module_api::ModuleServices;
    use bot_test_support::{FakeDiscord, FakeModuleServices, ServiceCall};
    use std::{collections::BTreeMap, sync::Arc};
    fn command(name: &str, id: u64, options: BTreeMap<String, CommandValue>) -> CommandInteraction {
        CommandInteraction {
            id,
            token: "t".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 40,
            actor_role_ids: vec![],
            actor_permissions: u64::MAX,
            command_name: name.into(),
            options,
        }
    }
    fn context() -> (ModuleContext, FakeDiscord, FakeModuleServices) {
        let d = FakeDiscord::new();
        let s = FakeModuleServices::default();
        (
            ModuleContext {
                discord: Arc::new(d.clone()),
                services: Arc::new(s.clone()),
            },
            d,
            s,
        )
    }
    #[tokio::test]
    async fn daily_is_cooldown_protected() {
        let (c, _, s) = context();
        let cfg = GuildConfig::for_guild("10");
        EconomyModule
            .on_command(&c, &cfg, &command("daily", 1, BTreeMap::new()))
            .await
            .unwrap();
        EconomyModule
            .on_command(&c, &cfg, &command("daily", 2, BTreeMap::new()))
            .await
            .unwrap();
        assert_eq!(s.economy_profile(10, 40).await.unwrap().balance, 250)
    }
    #[tokio::test]
    async fn transfers_are_atomic_and_self_pay_is_denied() {
        let (c, _, s) = context();
        s.economy_change_balance(10, 40, 1000, "seed", "seed")
            .await
            .unwrap();
        let options = BTreeMap::from([
            ("member".into(), CommandValue::User(41)),
            ("amount".into(), CommandValue::Integer(250)),
        ]);
        EconomyModule
            .on_command(
                &c,
                &GuildConfig::for_guild("10"),
                &command("pay", 2, options),
            )
            .await
            .unwrap();
        assert!(
            s.calls()
                .iter()
                .any(|call| matches!(call, ServiceCall::EconomyTransfer { amount: 250, .. }))
        );
        let self_options = BTreeMap::from([
            ("member".into(), CommandValue::User(40)),
            ("amount".into(), CommandValue::Integer(1)),
        ]);
        assert!(
            EconomyModule
                .on_command(
                    &c,
                    &GuildConfig::for_guild("10"),
                    &command("pay", 3, self_options)
                )
                .await
                .is_err()
        )
    }
    #[tokio::test]
    async fn games_enforce_wager_bounds() {
        let (c, d, s) = context();
        s.economy_change_balance(10, 40, 1000, "seed", "seed")
            .await
            .unwrap();
        let options = BTreeMap::from([
            ("wager".into(), CommandValue::Integer(1)),
            ("choice".into(), CommandValue::String("heads".into())),
        ]);
        assert!(
            EconomyModule
                .on_command(
                    &c,
                    &GuildConfig::for_guild("10"),
                    &command("coinflip", 4, options)
                )
                .await
                .is_err()
        );
        assert!(d.calls().is_empty())
    }
}
