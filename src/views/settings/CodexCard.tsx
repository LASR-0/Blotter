import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { ArrowsClockwise, CheckCircle, Code, WarningCircle } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { AI_SETTINGS_KEY, type AiSettings } from '@/lib/aiSettings';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';

interface CodexStatus {
  path: string | null;
  version: string | null;
  logged_in: boolean | null;
  /** `chatgpt` or `api_key`. */
  auth_method: string | null;
  error: string | null;
}

const STATUS_KEY = ['codex-status'];

const FIELD =
  'h-[30px] min-w-0 rounded-md border border-border-strong bg-bg px-2.5 font-mono text-sm text-text outline-none focus:border-accent';

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

function statusLine(s: CodexStatus): { ok: boolean; text: string } {
  if (s.error) return { ok: false, text: s.error };
  if (s.logged_in === false) {
    return { ok: false, text: 'Codex is installed but signed out. Run `codex login` to sign in.' };
  }
  const how =
    s.auth_method === 'chatgpt'
      ? 'your ChatGPT plan'
      : s.auth_method === 'api_key'
        ? 'an API key'
        : 'signed in';
  return { ok: true, text: `Codex ${s.version ?? ''} · signed in with ${how}` };
}

/** Settings → AI providers: the Codex route, driven like Claude Code. */
export function CodexCard({ settings }: { settings: AiSettings }) {
  const qc = useQueryClient();
  const status = useQuery({
    queryKey: STATUS_KEY,
    queryFn: () => invoke<CodexStatus>('codex_status'),
  });
  const [model, setModel] = useState(settings.codex_model);
  const [path, setPath] = useState(settings.codex_path);
  const [error, setError] = useState<string | null>(null);

  const save = (input: Partial<AiSettings>) =>
    invoke<AiSettings>('set_ai_settings', { input })
      .then((s) => qc.setQueryData(AI_SETTINGS_KEY, s))
      .catch((e: unknown) => setError(errorText(e)));
  const recheck = () => void qc.invalidateQueries({ queryKey: STATUS_KEY });

  const inUse = settings.route === 'codex';
  const line = status.data ? statusLine(status.data) : null;

  return (
    <div
      className={cn(
        'flex flex-col gap-3 rounded-lg border p-4',
        inUse ? 'border-border-strong' : 'border-border',
      )}
    >
      <div className="flex flex-wrap items-center gap-2">
        <Code size={15} className="text-accent" />
        <span className="text-md font-semibold">Codex</span>
        {inUse && (
          <span className="rounded-full bg-accent-tint px-2 text-xs text-accent-text">In use</span>
        )}
        <span className="ml-auto flex gap-1.5">
          <Button size="sm" variant="ghost" onClick={recheck} disabled={status.isFetching}>
            <ArrowsClockwise />
            Check again
          </Button>
          {!inUse && (
            <Button size="sm" onClick={() => void save({ route: 'codex' })}>
              Use for chat
            </Button>
          )}
        </span>
      </div>
      <p className="m-0 text-sm leading-normal text-text-muted">
        The chat runs through OpenAI&apos;s Codex CLI, installed and signed in on this computer, on
        your ChatGPT plan. It gets Blotter&apos;s tools only: its shell, files, web and apps are
        switched off.
      </p>

      {line && (
        <span
          className={cn(
            'flex items-center gap-1.5 text-sm',
            line.ok ? 'text-text-muted' : 'text-danger',
          )}
        >
          {line.ok ? (
            <CheckCircle size={13} className="text-tag-health" />
          ) : (
            <WarningCircle size={13} />
          )}
          {line.text}
        </span>
      )}

      <div className="grid grid-cols-[1fr_1fr] gap-3">
        <label className="flex flex-col gap-1.5 text-sm text-text-muted">
          Model
          <input
            value={model}
            onChange={(e) => setModel(e.target.value)}
            onBlur={() => {
              if (model !== settings.codex_model) void save({ codex_model: model });
            }}
            placeholder="Codex's default"
            aria-label="Codex model"
            className={FIELD}
          />
        </label>
        <label className="flex flex-col gap-1.5 text-sm text-text-muted">
          Program
          <input
            value={path}
            onChange={(e) => setPath(e.target.value)}
            onBlur={() => {
              if (path !== settings.codex_path) void save({ codex_path: path }).then(recheck);
            }}
            placeholder={status.data?.path ?? 'Found automatically'}
            aria-label="Codex program"
            className={FIELD}
          />
        </label>
      </div>
      {error && <p className="m-0 text-sm text-danger">{error}</p>}
    </div>
  );
}
