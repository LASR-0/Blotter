import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  CheckCircle,
  Flask,
  Globe,
  Key,
  LockKey,
  PlugsConnected,
  Plus,
  Trash,
  WarningCircle,
} from '@phosphor-icons/react';
import { useOverlay } from '@/app/overlay';
import { ErrorPanel } from '@/components/EmptyState';
import { Button } from '@/components/ui/button';
import { Select } from '@/components/ui/select';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';
import { AI_SETTINGS_KEY, type AiSettings } from '@/lib/aiSettings';
import { ClaudeCodeCard } from './ClaudeCodeCard';
import { McpSection } from './McpSection';

type Kind = 'anthropic' | 'openai' | 'openai_compatible';

interface ProviderView {
  id: string;
  name: string;
  kind: Kind;
  base_url: string | null;
  model: string;
  is_default: boolean;
  has_key: boolean;
  key_hint: string | null;
  experimental: boolean;
}

interface ModelInfo {
  id: string;
  display_name: string | null;
}

const PROVIDERS_KEY = ['ai-providers'];

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

const NEW: Record<Kind, { name: string; base_url: string | null; model: string }> = {
  anthropic: { name: 'Anthropic', base_url: null, model: 'claude-sonnet-5-5' },
  openai: { name: 'OpenAI', base_url: null, model: '' },
  openai_compatible: { name: 'Ollama', base_url: 'http://localhost:11434/v1', model: '' },
};

const FIELD =
  'h-[30px] min-w-0 rounded-md border border-border-strong bg-bg px-2.5 font-mono text-sm text-text outline-none focus:border-accent';

function ProviderCard({
  p,
  inUse,
  onUse,
  onChanged,
}: {
  p: ProviderView;
  /** The chat uses this provider (it is the default and the API route is on). */
  inUse: boolean;
  onUse: () => Promise<void>;
  onChanged: () => void;
}) {
  const ask = useOverlay((s) => s.ask);
  const [name, setName] = useState(p.name);
  const [baseUrl, setBaseUrl] = useState(p.base_url ?? '');
  const [model, setModel] = useState(p.model);
  const [key, setKey] = useState('');
  const [models, setModels] = useState<ModelInfo[] | null>(null);
  const [status, setStatus] = useState<{ ok: boolean; text: string } | null>(null);
  const [busy, setBusy] = useState(false);
  const needsKey = p.kind !== 'openai_compatible';
  const dirty =
    name !== p.name || baseUrl !== (p.base_url ?? '') || model !== p.model || key !== '';

  const save = async () => {
    setBusy(true);
    try {
      await invoke('save_provider', {
        input: {
          id: p.id,
          name,
          kind: p.kind,
          base_url: baseUrl.trim() || null,
          model,
          api_key: key ? key : null,
        },
      });
      setKey('');
      setStatus({ ok: true, text: 'Saved.' });
      onChanged();
    } catch (e) {
      setStatus({ ok: false, text: errorText(e) });
    } finally {
      setBusy(false);
    }
  };

  const run = async (what: 'test' | 'models') => {
    setBusy(true);
    setStatus(null);
    try {
      if (what === 'test') {
        const r = await invoke<{ ms: number; reply: string }>('test_provider', { id: p.id });
        setStatus({ ok: true, text: `Connected · ${r.ms} ms${r.reply ? ` · “${r.reply}”` : ''}` });
      } else {
        const list = await invoke<ModelInfo[]>('fetch_models', { id: p.id });
        setModels(list);
        setStatus({ ok: true, text: `${list.length} models available.` });
      }
    } catch (e) {
      setStatus({ ok: false, text: errorText(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <div
      className={cn(
        'flex flex-col gap-3 rounded-lg border p-4',
        inUse ? 'border-border-strong' : 'border-border',
      )}
    >
      <div className="flex flex-wrap items-center gap-2">
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          aria-label="Provider name"
          className="min-w-0 flex-1 bg-transparent text-md font-semibold outline-none"
        />
        {inUse && (
          <span className="rounded-full bg-accent-tint px-2 text-xs text-accent-text">In use</span>
        )}
        {p.experimental && (
          <span
            title="Built and unit-tested, but not yet tried against the real service."
            className="flex items-center gap-1 rounded-full border border-[color-mix(in_srgb,var(--color-warning)_45%,transparent)] px-2 text-xs text-warning"
          >
            <Flask size={11} />
            Experimental — untested
          </span>
        )}
      </div>

      <div className="grid grid-cols-[1fr_1fr] gap-3">
        {needsKey ? (
          <label className="flex flex-col gap-1.5 text-sm text-text-muted">
            <span className="flex items-center gap-1.5">
              <Key size={13} /> API key
            </span>
            <input
              type="password"
              value={key}
              onChange={(e) => setKey(e.target.value)}
              placeholder={p.has_key ? `••••••••${p.key_hint ?? ''}` : 'Paste key…'}
              aria-label="API key"
              autoComplete="off"
              className={FIELD}
            />
          </label>
        ) : (
          <label className="flex flex-col gap-1.5 text-sm text-text-muted">
            <span className="flex items-center gap-1.5">
              <Globe size={13} /> Base URL
            </span>
            <input
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              placeholder="http://localhost:11434/v1"
              aria-label="Base URL"
              className={FIELD}
            />
          </label>
        )}
        <label className="flex flex-col gap-1.5 text-sm text-text-muted">
          Model
          {models && models.length > 0 ? (
            <Select
              ariaLabel="Model"
              value={models.some((m) => m.id === model) ? model : ''}
              placeholder={model || 'Choose a model'}
              onValueChange={setModel}
              options={models.map((m) => ({ value: m.id, label: m.display_name ?? m.id }))}
              className="font-mono text-sm"
            />
          ) : (
            <input
              value={model}
              onChange={(e) => setModel(e.target.value)}
              placeholder="Model id, or Load models"
              aria-label="Model"
              className={FIELD}
            />
          )}
        </label>
      </div>

      {needsKey && p.has_key && (
        <span className="flex items-center gap-1.5 text-xs text-text-muted">
          <LockKey size={12} />
          Key stored in your system keychain. Never written to the database.
        </span>
      )}

      {status && (
        <span
          className={cn(
            'flex items-start gap-1.5 text-sm break-words',
            status.ok ? 'text-text-muted' : 'text-danger',
          )}
        >
          {status.ok ? (
            <CheckCircle size={14} className="mt-0.5 flex-none" />
          ) : (
            <WarningCircle size={14} className="mt-0.5 flex-none" />
          )}
          {status.text}
        </span>
      )}

      <div className="flex flex-wrap gap-1.5">
        <Button size="sm" variant="primary" disabled={!dirty || busy} onClick={() => void save()}>
          Save
        </Button>
        <Button size="sm" disabled={busy || dirty} onClick={() => void run('test')}>
          <PlugsConnected />
          Test connection
        </Button>
        <Button size="sm" disabled={busy || dirty} onClick={() => void run('models')}>
          Load models
        </Button>
        <span className="flex-1" />
        {!inUse && (
          <Button
            size="sm"
            variant="ghost"
            onClick={() =>
              onUse().catch((e: unknown) => setStatus({ ok: false, text: errorText(e) }))
            }
          >
            Use for chat
          </Button>
        )}
        <Button
          size="sm"
          variant="ghost"
          aria-label={`Remove ${p.name}`}
          onClick={() =>
            ask({
              title: `Remove ${p.name}?`,
              body: 'Its settings and saved key are deleted.',
              confirmLabel: 'Remove',
              danger: true,
              onConfirm: async () => {
                await invoke('delete_provider', { id: p.id });
                onChanged();
              },
            })
          }
        >
          <Trash />
        </Button>
      </div>
    </div>
  );
}

/** Settings → AI providers (design S4). The Propose/Auto switch arrives with #32. */
export function AiProvidersPage() {
  const qc = useQueryClient();
  const providers = useQuery({
    queryKey: PROVIDERS_KEY,
    queryFn: () => invoke<ProviderView[]>('list_providers'),
  });
  const [error, setError] = useState<string | null>(null);
  const ai = useQuery({
    queryKey: AI_SETTINGS_KEY,
    queryFn: () => invoke<AiSettings>('get_ai_settings'),
  });
  const refresh = () => void qc.invalidateQueries({ queryKey: PROVIDERS_KEY });
  const chooseProvider = async (id: string) => {
    await invoke('set_default_provider', { id });
    qc.setQueryData(
      AI_SETTINGS_KEY,
      await invoke<AiSettings>('set_ai_settings', { input: { route: 'api' } }),
    );
    refresh();
  };

  const add = (kind: Kind) =>
    invoke('save_provider', { input: { kind, ...NEW[kind] } })
      .then(refresh)
      .catch((e: unknown) => setError(errorText(e)));

  if (providers.isError || ai.isError) {
    return (
      <div className="px-8 pt-6">
        <ErrorPanel
          title="Couldn't load AI providers"
          message={errorText(providers.error ?? ai.error)}
        />
      </div>
    );
  }
  const list = providers.data ?? [];
  const has = (k: Kind) => list.some((p) => p.kind === k);

  return (
    <div className="flex max-w-[760px] flex-col gap-4 px-8 pt-6 pb-12">
      <h1 className="m-0 font-heading text-xl font-heading-weight">AI providers</h1>
      <div className="flex gap-2.5 rounded-md border border-border px-3.5 py-3 text-sm leading-normal text-text-muted">
        <Flask size={15} className="mt-px flex-none text-warning" />
        <span>
          Claude Code is the tested way to chat with Blotter. API-key providers are{' '}
          <strong className="font-medium text-text">experimental</strong>: they&apos;re built and
          tested against recorded responses, but haven&apos;t been tried against the real services
          yet, and calls are billed by the provider.
        </span>
      </div>
      {ai.data && <ClaudeCodeCard settings={ai.data} />}
      <h2 className="m-0 mt-2 text-md font-semibold">API-key providers</h2>
      {error && <p className="m-0 text-sm text-danger">{error}</p>}
      {list.map((p) => (
        <ProviderCard
          key={p.id}
          p={p}
          inUse={p.is_default && ai.data?.route === 'api'}
          onUse={() => chooseProvider(p.id)}
          onChanged={refresh}
        />
      ))}
      {providers.isSuccess && list.length === 0 && (
        <p className="m-0 text-sm text-text-muted">No providers yet. Add one below.</p>
      )}
      <div className="flex flex-wrap gap-2">
        {!has('anthropic') && (
          <Button onClick={() => void add('anthropic')}>
            <Plus />
            Add Anthropic
          </Button>
        )}
        {!has('openai') && (
          <Button onClick={() => void add('openai')}>
            <Plus />
            Add OpenAI
          </Button>
        )}
        <Button onClick={() => void add('openai_compatible')}>
          <Plus />
          Add OpenAI-compatible endpoint
        </Button>
      </div>
      <McpSection />
    </div>
  );
}
