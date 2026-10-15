import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { ArrowsClockwise, CheckCircle, Sparkle, WarningCircle } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { Select } from '@/components/ui/select';
import { AI_SETTINGS_KEY, type AiSettings } from '@/lib/aiSettings';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';

interface ClaudeCodeStatus {
  path: string | null;
  version: string | null;
  logged_in: boolean | null;
  auth_method: string | null;
  error: string | null;
}

const STATUS_KEY = ['claude-code-status'];

const MODELS = [
  { value: 'sonnet', label: 'Sonnet (latest)' },
  { value: 'opus', label: 'Opus (latest)' },
  { value: 'haiku', label: 'Haiku (latest)' },
  { value: 'fable', label: 'Fable (latest)' },
];

const FIELD =
  'h-[30px] min-w-0 rounded-md border border-border-strong bg-bg px-2.5 font-mono text-sm text-text outline-none focus:border-accent';

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

function statusLine(s: ClaudeCodeStatus): { ok: boolean; text: string } {
  if (s.error) return { ok: false, text: s.error };
  if (s.logged_in === false) {
    return {
      ok: false,
      text: 'Claude Code is installed but signed out. Run `claude` once to sign in.',
    };
  }
  const how =
    s.auth_method === 'claude.ai' ? 'your Claude subscription' : (s.auth_method ?? 'signed in');
  return { ok: true, text: `Claude Code ${s.version ?? ''} · signed in with ${how}` };
}

/** Settings → AI providers: the Claude Code route (D29), the primary way to chat. */
export function ClaudeCodeCard({ settings }: { settings: AiSettings }) {
  const qc = useQueryClient();
  const status = useQuery({
    queryKey: STATUS_KEY,
    queryFn: () => invoke<ClaudeCodeStatus>('claude_code_status'),
  });
  const [path, setPath] = useState(settings.claude_code_path);
  const [error, setError] = useState<string | null>(null);

  const save = (input: Partial<AiSettings>) =>
    invoke<AiSettings>('set_ai_settings', { input })
      .then((s) => qc.setQueryData(AI_SETTINGS_KEY, s))
      .catch((e: unknown) => setError(errorText(e)));
  const recheck = () => void qc.invalidateQueries({ queryKey: STATUS_KEY });

  const inUse = settings.route === 'claude_code';
  const line = status.data ? statusLine(status.data) : null;

  return (
    <div
      className={cn(
        'flex flex-col gap-3 rounded-lg border p-4',
        inUse ? 'border-border-strong' : 'border-border',
      )}
    >
      <div className="flex flex-wrap items-center gap-2">
        <Sparkle size={15} className="text-accent" />
        <span className="text-md font-semibold">Claude Code</span>
        {inUse && (
          <span className="rounded-full bg-accent-tint px-2 text-xs text-accent-text">In use</span>
        )}
        <span className="ml-auto flex gap-1.5">
          <Button size="sm" variant="ghost" onClick={recheck} disabled={status.isFetching}>
            <ArrowsClockwise />
            Check again
          </Button>
          {!inUse && (
            <Button size="sm" onClick={() => void save({ route: 'claude_code' })}>
              Use for chat
            </Button>
          )}
        </span>
      </div>
      <p className="m-0 text-sm leading-normal text-text-muted">
        The chat runs through your installed, signed-in Claude Code, on your existing subscription.
        It gets Blotter&apos;s tools only: no shell, files or web.
      </p>

      {line && (
        <span
          className={cn(
            'flex items-center gap-1.5 text-sm',
            line.ok ? 'text-text-muted' : 'text-danger',
          )}
        >
          {line.ok ? (
            <CheckCircle size={13} className="text-success" />
          ) : (
            <WarningCircle size={13} />
          )}
          {line.text}
        </span>
      )}

      <div className="grid grid-cols-[1fr_1fr] gap-3">
        <label className="flex flex-col gap-1.5 text-sm text-text-muted">
          Model
          <Select
            ariaLabel="Claude Code model"
            value={settings.claude_code_model}
            placeholder={settings.claude_code_model}
            onValueChange={(m) => void save({ claude_code_model: m })}
            options={
              MODELS.some((m) => m.value === settings.claude_code_model)
                ? MODELS
                : [
                    ...MODELS,
                    { value: settings.claude_code_model, label: settings.claude_code_model },
                  ]
            }
            className="text-sm"
          />
        </label>
        <label className="flex flex-col gap-1.5 text-sm text-text-muted">
          Program
          <input
            value={path}
            onChange={(e) => setPath(e.target.value)}
            onBlur={() => {
              if (path !== settings.claude_code_path)
                void save({ claude_code_path: path }).then(recheck);
            }}
            placeholder={status.data?.path ?? 'Found automatically'}
            aria-label="Claude Code program"
            className={FIELD}
          />
        </label>
      </div>
      {error && <p className="m-0 text-sm text-danger">{error}</p>}
    </div>
  );
}
