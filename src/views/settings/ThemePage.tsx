import { useState, type CSSProperties } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Check,
  Copy,
  DownloadSimple,
  PencilSimple,
  Trash,
  UploadSimple,
  Warning,
} from '@phosphor-icons/react';
import { useOverlay } from '@/app/overlay';
import { Button } from '@/components/ui/button';
import { Segmented } from '@/components/ui/segmented';
import { Select } from '@/components/ui/select';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';
import {
  AA_TEXT,
  contrastIssues,
  copyName,
  currentChoice,
  DENSITIES,
  formatRatio,
  HEADING_FONTS,
  RADII,
  readTokens,
  TEXT_SIZES,
  UI_FONTS,
  type Choice,
  type ContrastIssue,
} from '@/theme/engine';
import { PRESETS, TOKEN_GROUPS, type Theme, type ThemeVars, type TokenName } from '@/theme/presets';
import { useTheme } from '@/theme/theme';
import { ThemePreview } from './ThemePreview';

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

/** "--color-surface-raised" → "Surface raised". */
function tokenLabel(name: string): string {
  const words = name.replace(/^--(color|heat|tag|logo)-/, '').replace(/-/g, ' ');
  if (name.startsWith('--heat-')) return `Level ${words}`;
  if (name === '--tag-ics') return 'Calendars';
  return words.charAt(0).toUpperCase() + words.slice(1);
}

interface Draft {
  /** The user theme being edited; none for a new one. */
  id?: string;
  name: string;
  vars: ThemeVars;
}

function ThemeCard({
  theme,
  active,
  onPick,
}: {
  theme: Theme;
  active: boolean;
  onPick: () => void;
}) {
  return (
    <button
      type="button"
      role="radio"
      aria-checked={active}
      onClick={onPick}
      // The card renders in its own theme: tokens set on the element apply to its subtree.
      style={theme.vars as CSSProperties}
      className={cn(
        'flex cursor-pointer flex-col gap-2.5 rounded-lg border bg-bg p-4 text-left font-ui text-base text-text',
        active ? 'border-accent shadow-[0_0_0_1px_var(--color-accent)]' : 'border-border',
      )}
    >
      <span className="flex items-center">
        <span className="flex-1 truncate font-heading text-[15px] font-heading-weight">
          {theme.label}
        </span>
        {active && <Check weight="bold" className="text-accent-text" />}
      </span>
      <span className="flex flex-col gap-[3px] rounded-md border border-dashed border-pencil bg-[color-mix(in_srgb,var(--color-surface)_45%,transparent)] px-[11px] py-[9px]">
        <span className="font-medium">Call Mum</span>
        <span className="font-mono text-xs text-pencil">✎ proposed · 18:30</span>
      </span>
      <span className="flex flex-col gap-[3px] rounded-md border border-border bg-surface px-[11px] py-[9px]">
        <span className="font-medium">Gym</span>
        <span className="font-mono text-xs text-text-muted">17:30 · 45m · health</span>
      </span>
      <span className="h-[30px] rounded-sm bg-hatch px-1.5 py-[3px] font-mono text-[10px] text-text-muted">
        Work
      </span>
      <span className="truncate text-sm text-text-muted">{theme.note}</span>
    </button>
  );
}

function ColorRow({
  name,
  value,
  issue,
  onChange,
}: {
  name: TokenName;
  value: string;
  issue: ContrastIssue | undefined;
  onChange: (v: string) => void;
}) {
  const [text, setText] = useState(value);
  const [seen, setSeen] = useState(value);
  // Follow outside changes (another control, an import) unless being typed in.
  if (value !== seen) {
    setSeen(value);
    setText(value);
  }
  const valid = /^#[0-9a-f]{6}$/i.test(text);
  return (
    <label className="flex h-8 items-center gap-2.5">
      <input
        type="color"
        value={valid ? text : value}
        onChange={(e) => {
          setText(e.target.value);
          onChange(e.target.value);
        }}
        aria-label={`${tokenLabel(name)} colour`}
        className="h-6 w-8 flex-none cursor-pointer rounded-sm border border-border-strong bg-transparent p-0"
      />
      <span className="min-w-0 flex-1 truncate">{tokenLabel(name)}</span>
      {issue && (
        <span
          title={`${issue.label}: ${formatRatio(issue.ratio)}, below ${AA_TEXT}:1`}
          className="flex items-center gap-1 font-mono text-xs text-warning"
        >
          <Warning size={12} weight="fill" />
          {formatRatio(issue.ratio)}
        </span>
      )}
      <input
        value={text}
        onChange={(e) => {
          setText(e.target.value);
          if (/^#[0-9a-f]{6}$/i.test(e.target.value)) onChange(e.target.value.toLowerCase());
        }}
        onBlur={() => setText(value)}
        spellCheck={false}
        aria-label={`${tokenLabel(name)} hex`}
        aria-invalid={!valid}
        className={cn(
          'h-[26px] w-[84px] flex-none rounded-md border bg-bg px-2 font-mono text-sm text-text outline-none focus:border-accent',
          valid ? 'border-border-strong' : 'border-danger',
        )}
      />
    </label>
  );
}

function ChoiceRow({
  label,
  choices,
  vars,
  onPick,
  segmented,
}: {
  label: string;
  choices: readonly Choice[];
  vars: ThemeVars;
  onPick: (c: Choice) => void;
  segmented?: boolean;
}) {
  const current = currentChoice(choices, vars) ?? '';
  const pick = (id: string) => {
    const c = choices.find((x) => x.id === id);
    if (c) onPick(c);
  };
  return (
    <div className="flex min-h-8 items-center gap-3">
      <span className="w-[86px] flex-none text-text-muted">{label}</span>
      {segmented ? (
        <Segmented
          ariaLabel={label}
          value={current}
          onChange={pick}
          options={choices.map((c) => ({ value: c.id, label: c.label }))}
        />
      ) : (
        <Select
          ariaLabel={label}
          value={current}
          placeholder="Custom"
          onValueChange={pick}
          options={choices.map((c) => ({ value: c.id, label: c.label }))}
          className="min-w-[180px]"
        />
      )}
    </div>
  );
}

function Editor({
  draft,
  onChange,
  onDone,
}: {
  draft: Draft;
  onChange: (d: Draft) => void;
  onDone: () => void;
}) {
  const saveTheme = useTheme((s) => s.saveTheme);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const issues = contrastIssues(draft.vars);
  const issueOf = (name: TokenName) => issues.find((i) => i.fg === name || i.bg === name);
  const set = (vars: Partial<ThemeVars>) =>
    onChange({ ...draft, vars: { ...draft.vars, ...vars } as ThemeVars });

  const save = async (apply: boolean) => {
    setBusy(true);
    setError(null);
    try {
      const saved = await saveTheme(draft, apply);
      if (apply) onDone();
      else onChange({ ...draft, id: saved.id });
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-4 px-8 pt-6 pb-12">
      <div className="flex flex-wrap items-center gap-2">
        <input
          value={draft.name}
          onChange={(e) => onChange({ ...draft, name: e.target.value })}
          aria-label="Theme name"
          className="min-w-0 flex-1 bg-transparent font-heading text-xl font-heading-weight outline-none"
        />
        <Button
          onClick={() =>
            void invoke('export_theme', { name: draft.name, tokens: draft.vars }).catch((e) =>
              setError(errorText(e)),
            )
          }
        >
          <UploadSimple />
          Export…
        </Button>
        <Button variant="ghost" onClick={onDone}>
          Cancel
        </Button>
        <Button disabled={busy || !draft.name.trim()} onClick={() => void save(false)}>
          Save
        </Button>
        <Button
          variant="primary"
          disabled={busy || !draft.name.trim()}
          onClick={() => void save(true)}
        >
          Apply
        </Button>
      </div>
      <p className="m-0 max-w-[620px] leading-normal text-text-muted">
        Changes show in the preview only. Apply saves the theme and switches Blotter to it; Save
        keeps it for later.
      </p>
      {error && <p className="m-0 text-sm text-danger">{error}</p>}

      <div className="grid grid-cols-[minmax(300px,380px)_minmax(0,1fr)] items-start gap-6">
        <div className="flex flex-col gap-5">
          {issues.length > 0 ? (
            <div
              role="status"
              className="flex flex-col gap-1 rounded-lg border border-warning bg-[color-mix(in_srgb,var(--color-warning)_8%,var(--color-bg))] px-3 py-2.5 text-sm"
            >
              <span className="flex items-center gap-1.5 font-medium">
                <Warning weight="fill" className="text-warning" />
                {issues.length === 1
                  ? '1 pair is hard to read'
                  : `${issues.length} pairs are hard to read`}{' '}
                (below WCAG AA, {AA_TEXT}:1)
              </span>
              {issues.map((i) => (
                <span key={i.label} className="text-text-muted">
                  {i.label}: {formatRatio(i.ratio)}
                </span>
              ))}
            </div>
          ) : (
            <span className="flex items-center gap-1.5 text-sm text-text-muted">
              <Check /> Every text colour meets WCAG AA on its background.
            </span>
          )}

          <section className="flex flex-col gap-1.5">
            <h2 className="m-0 text-xs font-medium tracking-[.06em] text-text-muted uppercase">
              Shape and type
            </h2>
            <ChoiceRow
              label="Corners"
              choices={RADII}
              vars={draft.vars}
              onPick={(c) => set(c.vars)}
              segmented
            />
            <ChoiceRow
              label="Density"
              choices={DENSITIES}
              vars={draft.vars}
              onPick={(c) => set(c.vars)}
              segmented
            />
            <ChoiceRow
              label="Text size"
              choices={TEXT_SIZES}
              vars={draft.vars}
              onPick={(c) => set(c.vars)}
            />
            <ChoiceRow
              label="Font"
              choices={UI_FONTS}
              vars={draft.vars}
              onPick={(c) => set(c.vars)}
            />
            <ChoiceRow
              label="Headings"
              choices={HEADING_FONTS}
              vars={draft.vars}
              onPick={(c) => set(c.vars)}
            />
          </section>

          {TOKEN_GROUPS.map(([group, tokens]) => (
            <section key={group} className="flex flex-col">
              <h2 className="m-0 pb-1 text-xs font-medium tracking-[.06em] text-text-muted uppercase">
                {group}
              </h2>
              {tokens.map((t) => (
                <ColorRow
                  key={t}
                  name={t}
                  value={draft.vars[t] ?? '#000000'}
                  issue={issueOf(t)}
                  onChange={(v) => set({ [t]: v })}
                />
              ))}
            </section>
          ))}
        </div>
        <div className="sticky top-4">
          <ThemePreview vars={draft.vars} />
        </div>
      </div>
    </div>
  );
}

/** Settings → Theme (roadmap §5.5): presets and your themes, and the editor. */
export function ThemePage() {
  const { theme, userThemes, setTheme, deleteTheme } = useTheme();
  const ask = useOverlay((s) => s.ask);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const names = [...PRESETS, ...userThemes].map((t) => t.label);

  if (draft) return <Editor draft={draft} onChange={setDraft} onDone={() => setDraft(null)} />;

  const duplicate = (t: Theme) => setDraft({ name: copyName(t.label, names), vars: { ...t.vars } });
  const edit = (t: Theme) =>
    t.builtin ? duplicate(t) : setDraft({ id: t.id, name: t.label, vars: { ...t.vars } });

  const startImport = async () => {
    setError(null);
    try {
      const file = await invoke<{ name: string; tokens: Record<string, string> } | null>(
        'pick_theme_import',
      );
      if (!file) return;
      const { vars, dropped } = readTokens(file.tokens);
      setDraft({ name: file.name.trim() || 'Imported theme', vars });
      if (dropped > 0) setStatus(`${dropped} tokens this version doesn't use were left out.`);
    } catch (e) {
      setError(errorText(e));
    }
  };

  const all = [...PRESETS, ...userThemes];
  return (
    <div className="flex max-w-[960px] flex-col gap-4 px-8 pt-6 pb-12">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="m-0 flex-1 font-heading text-xl font-heading-weight">Theme</h1>
        <Button onClick={() => void startImport()}>
          <DownloadSimple />
          Import…
        </Button>
        <Button variant="primary" onClick={() => edit(theme)}>
          <PencilSimple />
          {theme.builtin ? 'Customise' : 'Edit'} {theme.label}
        </Button>
      </div>
      <p className="m-0 max-w-[600px] leading-normal text-text-muted">
        Pick a theme to use it. Customise a preset to make your own copy: edit colours, corners,
        type and density with a live preview, and export it to share.
      </p>
      {status && <p className="m-0 text-sm text-text-muted">{status}</p>}
      {error && <p className="m-0 text-sm text-danger">{error}</p>}
      <div role="radiogroup" aria-label="Theme" className="grid grid-cols-4 gap-3.5">
        {all.map((t) => (
          <div key={t.id} className="flex flex-col gap-1.5">
            <ThemeCard theme={t} active={theme.id === t.id} onPick={() => setTheme(t.id)} />
            <div className="flex justify-end gap-0.5">
              <Button
                size="icon-sm"
                variant="bare"
                aria-label={`${t.builtin ? 'Customise a copy of' : 'Edit'} ${t.label}`}
                title={t.builtin ? 'Customise a copy' : 'Edit'}
                onClick={() => edit(t)}
              >
                <PencilSimple />
              </Button>
              <Button
                size="icon-sm"
                variant="bare"
                aria-label={`Duplicate ${t.label}`}
                title="Duplicate"
                onClick={() => duplicate(t)}
              >
                <Copy />
              </Button>
              <Button
                size="icon-sm"
                variant="bare"
                aria-label={`Export ${t.label}`}
                title="Export…"
                onClick={() =>
                  void invoke<string | null>('export_theme', { name: t.label, tokens: t.vars })
                    .then((path) => path && setStatus(`Saved ${t.label} to ${path}`))
                    .catch((e: unknown) => setError(errorText(e)))
                }
              >
                <UploadSimple />
              </Button>
              {!t.builtin && (
                <Button
                  size="icon-sm"
                  variant="bare"
                  aria-label={`Delete ${t.label}`}
                  title="Delete"
                  onClick={() =>
                    ask({
                      title: `Delete ${t.label}?`,
                      body:
                        theme.id === t.id
                          ? 'Blotter switches to Dark. Export it first to keep a copy.'
                          : 'Export it first to keep a copy.',
                      confirmLabel: 'Delete',
                      danger: true,
                      onConfirm: () => deleteTheme(t.id),
                    })
                  }
                >
                  <Trash />
                </Button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
