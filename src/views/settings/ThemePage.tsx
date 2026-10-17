import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Copy, DownloadSimple, FileArrowDown, Trash, Warning } from '@phosphor-icons/react';
import { useOverlay } from '@/app/overlay';
import { Button } from '@/components/ui/button';
import { Segmented } from '@/components/ui/segmented';
import { Select } from '@/components/ui/select';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';
import {
  AA_TEXT,
  baseRadius,
  contrastIssues,
  copyName,
  currentChoice,
  DENSITY,
  HEADINGS,
  radiusVars,
  readTokens,
  TEXT_SIZES,
  UI_FONTS,
  type Choice,
  type ContrastIssue,
} from '@/theme/engine';
import {
  DEFAULT_THEME_ID,
  findPreset,
  PRESETS,
  TOKEN_GROUPS,
  type Theme,
  type ThemeVars,
  type TokenName,
} from '@/theme/presets';
import { useTheme } from '@/theme/theme';
import { ThemePreview } from './ThemePreview';

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));
const HEX = /^#[0-9a-f]{6}$/i;

/** The draft being edited: a copy of `base` until saved. */
interface Draft {
  base: Theme;
  name: string;
  vars: ThemeVars;
}

/** Blotter Dark: where Discard and a deleted theme fall back to. */
const DEFAULT_PRESET = (findPreset(DEFAULT_THEME_ID) ?? PRESETS[0]) as Theme;
const fromTheme = (t: Theme): Draft => ({ base: t, name: t.label, vars: { ...t.vars } });
const sameVars = (a: ThemeVars, b: ThemeVars) =>
  Object.keys(a).length === Object.keys(b).length &&
  Object.entries(a).every(([k, v]) => b[k as TokenName] === v);

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
  // Follow outside changes (a preset, an import) unless being typed in.
  if (value !== seen) {
    setSeen(value);
    setText(value);
  }
  const valid = HEX.test(text);
  const label = name.replace(/^--/, '');
  return (
    <div className="flex flex-col gap-[3px] py-[3px]">
      <div className="flex items-center gap-2">
        <input
          type="color"
          value={valid ? text : value}
          onChange={(e) => {
            setText(e.target.value);
            onChange(e.target.value);
          }}
          aria-label={`${label} colour`}
          className="size-[22px] flex-none cursor-pointer rounded-[4px] border border-border-strong bg-transparent p-0"
        />
        <span className="min-w-0 flex-1 truncate font-mono text-xs text-text-muted">{label}</span>
        <input
          value={text}
          onChange={(e) => {
            setText(e.target.value);
            if (HEX.test(e.target.value)) onChange(e.target.value.toLowerCase());
          }}
          onBlur={() => setText(value)}
          spellCheck={false}
          aria-label={`${label} hex`}
          aria-invalid={!valid}
          className={cn(
            'h-6 w-[84px] flex-none rounded-[4px] border bg-bg px-1.5 font-mono text-xs text-text outline-none focus:border-accent',
            !valid ? 'border-danger' : issue ? 'border-warning' : 'border-border-strong',
          )}
        />
      </div>
      {issue && (
        <span className="flex items-center gap-[5px] pl-[30px] text-xs text-warning">
          <Warning weight="fill" />
          {issue.ratio.toFixed(2)}:1 on {issue.bg.replace(/^--color-/, '').replace(/-/g, ' ')} — AA
          needs {AA_TEXT}:1
        </span>
      )}
    </div>
  );
}

function ChoiceRow({
  label,
  choices,
  vars,
  onPick,
  select,
}: {
  label: string;
  choices: readonly Choice[];
  vars: ThemeVars;
  onPick: (c: Choice) => void;
  select?: boolean;
}) {
  const current = currentChoice(choices, vars) ?? '';
  const pick = (id: string) => {
    const c = choices.find((x) => x.id === id);
    if (c) onPick(c);
  };
  const options = choices.map((c) => ({ value: c.id, label: c.label }));
  return (
    <div className="flex items-center gap-2.5 text-sm text-text-muted">
      <span className="w-[70px] flex-none">{label}</span>
      {select ? (
        <Select
          ariaLabel={label}
          value={current}
          placeholder="Custom"
          onValueChange={pick}
          options={options}
          className="min-w-[150px] text-sm"
        />
      ) : (
        <Segmented ariaLabel={label} value={current} onChange={pick} options={options} />
      )}
    </div>
  );
}

/**
 * Settings → Theme (roadmap §5.5, design: Settings Theme): one draft, a
 * token editor beside a live preview. Only the preview uses the draft
 * until Apply.
 */
export function ThemePage() {
  const { theme, userThemes, saveTheme, deleteTheme, setTheme } = useTheme();
  const ask = useOverlay((s) => s.ask);
  const [draft, setDraft] = useState<Draft>(() => fromTheme(theme));
  const [applied, setApplied] = useState(false);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const edited = !sameVars(draft.vars, draft.base.vars) || draft.name !== draft.base.label;
  const issues = contrastIssues(draft.vars);
  const issueOf = (t: TokenName) => issues.find((i) => i.fg === t);
  const names = [...PRESETS, ...userThemes].map((t) => t.label);

  const change = (d: Draft) => {
    setDraft(d);
    setApplied(false);
    setStatus(null);
  };
  const set = (vars: Partial<ThemeVars>) =>
    change({ ...draft, vars: { ...draft.vars, ...vars } as ThemeVars });

  /** Saves the draft (a preset's edits become a theme of your own), optionally applying it. */
  const save = async (apply: boolean) => {
    setBusy(true);
    setError(null);
    try {
      if (draft.base.builtin && !edited) {
        if (apply) setTheme(draft.base.id);
      } else {
        const name =
          draft.base.builtin && draft.name === draft.base.label
            ? copyName(draft.name, names)
            : draft.name.trim() || 'My theme';
        const saved = await saveTheme(
          { id: draft.base.builtin ? undefined : draft.base.id, name, vars: draft.vars },
          apply,
        );
        setDraft(fromTheme(saved));
      }
      setApplied(apply);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  // Ctrl+Enter applies, as the design's hint says.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey) && !busy) {
        e.preventDefault();
        void save(true);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  const startImport = async () => {
    setError(null);
    try {
      const file = await invoke<{ name: string; tokens: Record<string, string> } | null>(
        'pick_theme_import',
      );
      if (!file) return;
      const { vars, dropped } = readTokens(file.tokens);
      // Saved as a theme of your own on Apply; Discard goes back to Blotter Dark.
      change({ base: DEFAULT_PRESET, name: file.name.trim() || 'Imported theme', vars });
      if (dropped > 0) setStatus(`${dropped} tokens this version doesn't use were left out.`);
    } catch (e) {
      setError(errorText(e));
    }
  };

  const presetChip = (t: Theme) => (
    <button
      key={t.id}
      type="button"
      aria-pressed={draft.base.id === t.id}
      onClick={() => change(fromTheme(t))}
      className={cn(
        'flex min-w-0 cursor-pointer flex-col gap-[5px] rounded-md border bg-transparent p-1.5 text-left text-xs text-text',
        draft.base.id === t.id ? 'border-accent' : 'border-border',
      )}
    >
      <span className="flex h-[18px] overflow-hidden rounded-[3px] border border-border">
        <span className="flex-[2]" style={{ background: t.vars['--color-bg'] }} />
        <span className="flex-1" style={{ background: t.vars['--color-accent'] }} />
      </span>
      <span className="truncate">{t.label}</span>
    </button>
  );

  return (
    <div className="@container/theme flex h-full min-h-0 flex-col">
      <div className="flex flex-wrap items-center gap-2.5 border-b border-border px-5 py-3.5">
        <h1 className="m-0 font-heading text-xl font-heading-weight">Theme</h1>
        <label className="flex items-center gap-1 rounded-full border border-dashed border-pencil px-2 py-0.5 font-mono text-xs whitespace-nowrap text-text-muted">
          draft ·
          <input
            value={draft.name}
            onChange={(e) => change({ ...draft, name: e.target.value })}
            aria-label="Theme name"
            size={Math.max(6, draft.name.length)}
            className="bg-transparent text-text-muted outline-none focus:text-text"
          />
          {edited && <span>(edited)</span>}
        </label>
        {issues.length > 0 && (
          <span className="flex items-center gap-[5px] text-sm whitespace-nowrap text-warning">
            <Warning weight="fill" />
            {issues.length} below WCAG AA
          </span>
        )}
        <span className="flex-1" />
        <Button variant="ghost" onClick={() => void startImport()}>
          <DownloadSimple />
          Import
        </Button>
        <Button
          onClick={() =>
            change({
              base: { ...draft.base, builtin: true, label: draft.name },
              name: copyName(draft.name, names),
              vars: { ...draft.vars },
            })
          }
        >
          <Copy />
          Duplicate
        </Button>
        <Button
          onClick={() =>
            void invoke<string | null>('export_theme', { name: draft.name, tokens: draft.vars })
              .then((path) => path && setStatus(`Saved ${draft.name} to ${path}`))
              .catch((e: unknown) => setError(errorText(e)))
          }
        >
          <FileArrowDown />
          Export
        </Button>
        {!draft.base.builtin && (
          <Button
            variant="ghost"
            aria-label={`Delete ${draft.base.label}`}
            title="Delete this theme"
            onClick={() =>
              ask({
                title: `Delete ${draft.base.label}?`,
                body:
                  theme.id === draft.base.id
                    ? 'Blotter switches to Blotter Dark. Export it first to keep a copy.'
                    : 'Export it first to keep a copy.',
                confirmLabel: 'Delete',
                danger: true,
                onConfirm: async () => {
                  await deleteTheme(draft.base.id);
                  change(fromTheme(DEFAULT_PRESET));
                },
              })
            }
          >
            <Trash />
          </Button>
        )}
        <Button variant="ghost" disabled={!edited} onClick={() => change(fromTheme(draft.base))}>
          Discard
        </Button>
        <Button variant="primary" disabled={busy} onClick={() => void save(true)}>
          {applied ? 'Applied' : 'Apply'}
          <span className="rounded-[3px] bg-[color-mix(in_srgb,var(--color-on-accent)_18%,transparent)] px-1 font-mono text-[10px]">
            Ctrl ↵
          </span>
        </Button>
      </div>
      {(status || error) && (
        <p className={cn('m-0 px-5 pt-2 text-sm', error ? 'text-danger' : 'text-text-muted')}>
          {error ?? status}
        </p>
      )}

      <div className="flex min-h-0 flex-1 flex-col overflow-auto @min-[760px]/theme:flex-row @min-[760px]/theme:overflow-visible">
        <div className="flex flex-none flex-col gap-4 border-b border-border px-4 pt-3.5 pb-8 @min-[760px]/theme:w-[340px] @min-[760px]/theme:overflow-auto @min-[760px]/theme:border-r @min-[760px]/theme:border-b-0">
          <div className="flex flex-col gap-1.5">
            <span className="text-sm text-text-muted">Start from preset</span>
            <div className="grid grid-cols-4 gap-1.5">{PRESETS.map(presetChip)}</div>
            {userThemes.length > 0 && (
              <>
                <span className="pt-1 text-sm text-text-muted">Your themes</span>
                <div className="grid grid-cols-4 gap-1.5">{userThemes.map(presetChip)}</div>
              </>
            )}
          </div>

          {TOKEN_GROUPS.map(([group, tokens]) => (
            <div key={group} className="flex flex-col gap-0.5">
              <span className="pb-1 text-xs tracking-[.06em] text-text-muted uppercase">
                {group}
              </span>
              {tokens.map((t) => (
                <ColorRow
                  key={t}
                  name={t}
                  value={draft.vars[t] ?? '#000000'}
                  issue={issueOf(t)}
                  onChange={(v) => set({ [t]: v })}
                />
              ))}
            </div>
          ))}

          <div className="flex flex-col gap-2.5">
            <span className="text-xs tracking-[.06em] text-text-muted uppercase">
              Shape &amp; type
            </span>
            <label className="flex items-center gap-2.5 text-sm text-text-muted">
              <span className="w-[70px]">Radius</span>
              <input
                type="range"
                min={0}
                max={12}
                value={baseRadius(draft.vars)}
                onChange={(e) => set(radiusVars(Number(e.target.value)))}
                className="flex-1 accent-accent"
              />
              <span className="w-[34px] text-right font-mono text-text">
                {baseRadius(draft.vars)}px
              </span>
            </label>
            <ChoiceRow
              label="Headings"
              choices={HEADINGS}
              vars={draft.vars}
              onPick={(c) => set(c.vars)}
            />
            <ChoiceRow
              label="Density"
              choices={DENSITY}
              vars={draft.vars}
              onPick={(c) => set(c.vars)}
            />
            <ChoiceRow
              label="Text size"
              choices={TEXT_SIZES}
              vars={draft.vars}
              onPick={(c) => set(c.vars)}
              select
            />
            <ChoiceRow
              label="Font"
              choices={UI_FONTS}
              vars={draft.vars}
              onPick={(c) => set(c.vars)}
              select
            />
          </div>
        </div>

        <div className="flex min-w-0 flex-none flex-col @min-[760px]/theme:flex-1 @min-[760px]/theme:overflow-auto">
          <div className="px-[18px] pt-3.5 text-sm text-text-muted">
            Live preview
            <span className="ml-1.5">· only this panel uses the draft until you apply</span>
          </div>
          <div className="p-[18px] pt-2.5">
            <ThemePreview vars={draft.vars} />
          </div>
        </div>
      </div>
    </div>
  );
}
