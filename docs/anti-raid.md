# Anti-raid design

Orivane treats anti-raid as layered containment, not as a replacement for Discord's native safety controls.

## Comparison

| System | Signals | Response model | Orivane decision |
| --- | --- | --- | --- |
| [Discord Raid Protection](https://support.discord.com/hc/en-us/articles/10989121220631-How-to-Protect-Your-Server-from-Raids-101) | Discord-controlled machine-learning signals | Staff alert and native CAPTCHA for new joiners; Discord also recommends stronger verification and mention limits | Complement it. Bots cannot enable Discord's private CAPTCHA system, so the dashboard should tell owners to enable it separately. |
| [Wick Join Gate and Join Raid](https://docs.wick.bot/intro/features/) | Join bursts, account age, missing avatars, bot additions, usernames and other join filters | Per-filter timeout/kick/ban/log actions, staff warnings, verification and lockdown | Use join bursts plus account risk, reversible quarantine by default, staff alerts, and manual join lockdown. |
| [Security Bot Anti-Raid](https://docs.securitybot.gg/other-modules/anti-raid) | Score based on account age, join speed, avatar and normal joins | Threshold activates punishment and optional server lock; bypasses reduce false positives | Keep signals explainable and configurable. Current Orivane uses explicit burst and risk thresholds rather than an opaque score. |
| [Beemo](https://docs.beemo.gg/) | Known userbot/spam raid behavior | Automatic anti-spam plus a manual raid-report workflow | Keep message AutoMod active as the second layer after join containment. |
| [Double Counter](https://docs.doublecounter.gg/double-counter-en/alt-detections-and-vpn-intrusions) | Web verification, IP/device relationships and VPN/proxy detection | Verification role, timeouts, alt decisions and manual verification | Not copied into core anti-raid: it requires collection of network/device data and creates privacy and false-positive tradeoffs. A separate opt-in verification product would need its own review. |

## Implemented behavior

- A Redis-backed sliding join window shared by every gateway worker.
- Risk signals for new accounts, missing avatars and bot accounts.
- Optional containment of every join during an incident.
- Alert-only, quarantine-role, timeout, kick and ban actions.
- Durable moderation cases and audit events for caught members.
- Optional staff alert channel.
- `/raid-lockdown minutes:<1-1440>` and `/raid-unlock`, restricted to members with Manage Server by Discord's command permission system.
- Redis-backed lockdown expiry so restarts and multiple shards agree on state.

## Recommended starting policy

Use a 10-join/60-second window, seven-day minimum account age, quarantine action, and a private staff alert channel. Enable Discord's native Raid Protection and at least Low verification separately. Review incidents before moving from quarantine to kick or ban.

The bot role must be above the quarantine role and must have Manage Roles for quarantine, Moderate Members for timeout, Kick Members for kick, or Ban Members for ban.
