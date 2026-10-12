import type { ColumnDef } from "@tanstack/react-table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useParams } from "@tanstack/react-router";
import { ArrowLeft, Check, LoaderCircle, Plus, Trash2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { DataTable } from "../components/data-table";
import { DiscordChannelSelect } from "../components/discord-channel-select";
import { DiscordRoleSelect } from "../components/discord-role-select";
import { MetricChart } from "../components/metric-chart";
import { Badge, Button, Panel, SectionHeader, Toggle } from "../components/ui";
import {
  getGuildChannels,
  getGuildConfig,
  getGuildRoles,
  getMessageDeletion,
  getMessageDeletions,
  saveGuildConfig,
} from "../lib/api";
import type { AutoDeleteRule, GuildConfig, MessageDeletionJob } from "../types";
export function AutoDeletePage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string };
  const qc = useQueryClient(),
    key = ["guild-config", serverId] as const;
  const config = useQuery({
      queryKey: key,
      queryFn: () => getGuildConfig(serverId),
    }),
    jobs = useQuery({
      queryKey: ["message-deletions", serverId],
      queryFn: () => getMessageDeletions(serverId),
    }),
    channels = useQuery({
      queryKey: ["guild-channels", serverId],
      queryFn: () => getGuildChannels(serverId),
    }),
    roles = useQuery({
      queryKey: ["guild-roles", serverId],
      queryFn: () => getGuildRoles(serverId),
    });
  const [draft, setDraft] = useState<GuildConfig | null>(null);
  useEffect(() => setDraft(config.data ?? null), [config.data, serverId]);
  const save = useMutation({
    mutationFn: saveGuildConfig,
    onSuccess: (v) => {
      qc.setQueryData(key, v);
      setDraft(v);
    },
  });
  const columns = useMemo<Array<ColumnDef<MessageDeletionJob>>>(
    () => [
      {
        accessorKey: "message_id",
        header: "Message",
        cell: ({ row }) => <code>{row.original.message_id}</code>,
      },
      { accessorKey: "rule_id", header: "Rule" },
      {
        accessorKey: "status",
        header: "Status",
        cell: ({ row }) => (
          <Badge
            tone={
              row.original.status === "deleted"
                ? "green"
                : row.original.status === "failed"
                  ? "red"
                  : "violet"
            }
          >
            {row.original.status}
          </Badge>
        ),
      },
      { accessorKey: "attempts", header: "Attempts" },
      {
        accessorKey: "due_at",
        header: "Due",
        cell: ({ row }) => format(row.original.due_at),
      },
    ],
    [],
  );
  if (!draft)
    return (
      <div className="loading-state">
        <LoaderCircle className="spin" />
        Loading auto-delete…
      </div>
    );
  const mod = draft.modules.auto_delete,
    items = jobs.data ?? [],
    changed = JSON.stringify(config.data) !== JSON.stringify(draft),
    update = (rules: AutoDeleteRule[]) =>
      setDraft({
        ...draft,
        modules: { ...draft.modules, auto_delete: { ...mod, rules } },
      });
  const chart = ["scheduled", "deleted", "failed"].map((status) => ({
    label: status,
    value: items.filter((x) => x.status === status).length,
    detail: `${items.filter((x) => x.status === status).length} jobs are ${status}.`,
  }));
  return (
    <div className="page-stack auto-delete-page">
      <Link
        className="back-link"
        to="/servers/$serverId/modules"
        params={{ serverId }}
      >
        <ArrowLeft size={16} />
        All modules
      </Link>
      <section className="page-intro">
        <div>
          <p className="eyebrow">Message lifecycle</p>
          <h2>Auto-delete</h2>
          <p>
            Remove messages after a channel-specific delay with durable retries.
            Message contents are never stored.
          </p>
        </div>
        <Button
          onClick={() =>
            update([
              ...mod.rules,
              {
                enabled: true,
                id: `rule-${mod.rules.length + 1}`,
                channel_id: "",
                delay_seconds: 60,
                ignore_bot_messages: true,
                excluded_role_ids: [],
              },
            ])
          }
        >
          <Plus size={16} />
          Add rule
        </Button>
      </section>
      <Panel className="form-detail-panel">
        <SectionHeader
          eyebrow="Rules"
          title="Channel deletion rules"
          detail="Each matching message is queued by Discord ID only."
        />
        <div className="section-body custom-command-editor">
          {mod.rules.map((rule, index) => (
            <Rule
              key={`${rule.id}-${index}`}
              rule={rule}
              channels={channels.data ?? []}
              roles={roles.data ?? []}
              pending={channels.isLoading || roles.isLoading}
              onChange={(value) =>
                update(mod.rules.map((r, i) => (i === index ? value : r)))
              }
              onRemove={() => update(mod.rules.filter((_, i) => i !== index))}
            />
          ))}
          {!mod.rules.length && (
            <div className="quiet-state">
              <Trash2 />
              <strong>No deletion rules</strong>
              <span>Add a rule to begin cleaning a channel.</span>
            </div>
          )}
        </div>
      </Panel>
      <Panel className="analytics-chart-panel">
        <SectionHeader
          eyebrow="Queue analytics"
          title="Deletion jobs by status"
        />
        <MetricChart
          data={chart}
          xLabel="Deletion status"
          yLabel="Jobs"
          ariaLabel="Message deletion jobs grouped by status"
          type="bar"
          color="#ef4444"
        />
      </Panel>
      <Panel className="table-panel">
        <SectionHeader
          eyebrow="Content-safe queue"
          title="Recent deletion jobs"
        />
        <DataTable
          data={items}
          columns={columns}
          getRowId={(r) => `${r.channel_id}-${r.message_id}`}
          ariaLabel="Message deletion jobs"
          emptyMessage="No deletion jobs yet"
        />
      </Panel>
      <div className="bottom-save-bar">
        <div className="module-save-toggle">
          <Toggle
            label="Enable Auto-delete"
            checked={mod.enabled}
            onChange={(enabled) =>
              setDraft({
                ...draft,
                modules: { ...draft.modules, auto_delete: { ...mod, enabled } },
              })
            }
          />
          <span>
            <strong>Auto-delete module</strong>
              <small>Schedules content-safe message deletion rules.</small>
          </span>
        </div>
        <div className="save-row">
          {save.isSuccess && !changed && (
            <span className="saved-label">
              <Check size={15} />
              Saved
            </span>
          )}
          <Button
            disabled={!changed || save.isPending}
            onClick={() => save.mutate(draft)}
          >
            Save changes
          </Button>
        </div>
      </div>
    </div>
  );
}
function Rule({
  rule,
  channels,
  roles,
  pending,
  onChange,
  onRemove,
}: {
  rule: AutoDeleteRule;
  channels: any[];
  roles: any[];
  pending: boolean;
  onChange: (v: AutoDeleteRule) => void;
  onRemove: () => void;
}) {
  return (
    <div className="custom-command-card">
      <div className="custom-command-card-head">
        <Toggle
          label={`Enable ${rule.id}`}
          checked={rule.enabled}
          onChange={(enabled) => onChange({ ...rule, enabled })}
        />
        <Button onClick={onRemove}>
          <Trash2 size={16} />
          Remove
        </Button>
      </div>
      <div className="form-grid">
        <label>
          <span>Rule ID</span>
          <small>Unique audit identifier</small>
          <input
            value={rule.id}
            onChange={(e) =>
              onChange({
                ...rule,
                id: e.target.value.toLowerCase().replace(/[^a-z0-9_-]/g, ""),
              })
            }
          />
        </label>
        <DiscordChannelSelect
          title="Channel"
          detail="Messages queued for deletion"
          value={rule.channel_id || null}
          kind="text"
          channels={channels}
          pending={pending}
          error={false}
          onChange={(channel_id) =>
            onChange({ ...rule, channel_id: channel_id ?? "" })
          }
        />
        <label>
          <span>Delete after</span>
          <small>Seconds, from 5 to 30 days</small>
          <input
            type="number"
            min={5}
            max={2592000}
            value={rule.delay_seconds}
            onChange={(e) =>
              onChange({ ...rule, delay_seconds: Number(e.target.value) })
            }
          />
        </label>
        <DiscordRoleSelect
          title="Excluded roles"
          detail="Members with these roles are retained"
          values={rule.excluded_role_ids}
          roles={roles}
          pending={pending}
          error={false}
          max={25}
          onChange={(excluded_role_ids) =>
            onChange({ ...rule, excluded_role_ids })
          }
        />
        <div className="module-toggle-field">
          <span>
            <strong>Ignore bot messages</strong>
            <small>Keep messages written by bots</small>
          </span>
          <Toggle
            label="Ignore bot messages"
            checked={rule.ignore_bot_messages}
            onChange={(ignore_bot_messages) =>
              onChange({ ...rule, ignore_bot_messages })
            }
          />
        </div>
      </div>
    </div>
  );
}
function format(v: string) {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(v));
}

export function AutoDeleteJobPage() {
  const { serverId, channelId, messageId } = useParams({ strict: false }) as {
    serverId: string;
    channelId: string;
    messageId: string;
  };
  const job = useQuery({
    queryKey: ["message-deletion", serverId, channelId, messageId],
    queryFn: () => getMessageDeletion(serverId, channelId, messageId),
  });
  if (job.isLoading)
    return (
      <div className="loading-state">
        <LoaderCircle className="spin" />
        Loading deletion job…
      </div>
    );
  if (!job.data)
    return (
      <div className="empty-state">
        <h2>Deletion job not found</h2>
      </div>
    );
  const item = job.data;
  return (
    <div className="page-stack">
      <Link
        className="back-link"
        to="/servers/$serverId/modules/auto_delete"
        params={{ serverId }}
      >
        <ArrowLeft size={16} />
        Auto-delete
      </Link>
      <section className="page-intro">
        <div>
          <p className="eyebrow">Deletion job</p>
          <h2>Message {item.message_id}</h2>
          <p>Content-safe lifecycle record for channel {item.channel_id}.</p>
        </div>
        <Badge
          tone={
            item.status === "deleted"
              ? "green"
              : item.status === "failed"
                ? "red"
                : "violet"
          }
        >
          {item.status}
        </Badge>
      </section>
      <Panel className="giveaway-rules">
        <SectionHeader eyebrow="Lifecycle" title="Job details" />
        <dl className="section-body">
          <div>
            <dt>Rule</dt>
            <dd>{item.rule_id}</dd>
          </div>
          <div>
            <dt>Author ID</dt>
            <dd>{item.author_id}</dd>
          </div>
          <div>
            <dt>Due</dt>
            <dd>{format(item.due_at)}</dd>
          </div>
          <div>
            <dt>Attempts</dt>
            <dd>{item.attempts} / 5</dd>
          </div>
          <div>
            <dt>Deleted</dt>
            <dd>{item.deleted_at ? format(item.deleted_at) : "Not deleted"}</dd>
          </div>
          <div>
            <dt>Source</dt>
            <dd>{item.source}</dd>
          </div>
        </dl>
      </Panel>
    </div>
  );
}
