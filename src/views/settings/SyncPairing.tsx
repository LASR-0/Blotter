import { useState, type InputHTMLAttributes } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';
import type { JoinChoiceValue, Joined } from '@/lib/syncFormat';

// Pairing, shared by Settings → Sync and first run's "Connect to a home
// server" (roadmap §14.7).

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

export const FIELD =
  'h-[30px] min-w-0 rounded-md border border-border-strong bg-bg px-2.5 text-sm text-text outline-none focus:border-accent';

/** Address, pairing code and this device's name, traded for a token. */
export function PairForm({
  again,
  bare,
  address: initialAddress,
  name: initialName,
  onPaired,
}: {
  again?: boolean;
  /** Without its own card and heading (first run has both). */
  bare?: boolean;
  address?: string;
  name?: string | null;
  onPaired: (joined: Joined) => void;
}) {
  const [address, setAddress] = useState(initialAddress ?? '');
  const [code, setCode] = useState('');
  const [name, setName] = useState(initialName ?? '');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const ready = address.trim() && code.trim() && name.trim();

  const pair = async () => {
    setBusy(true);
    setError(null);
    try {
      const joined = await invoke<Joined>('sync_pair', { address, code, name });
      setCode('');
      onPaired(joined);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const field = (
    label: string,
    value: string,
    set: (v: string) => void,
    props: InputHTMLAttributes<HTMLInputElement>,
  ) => (
    <label className="flex flex-col gap-1 text-sm text-text-muted">
      {label}
      <input
        value={value}
        onChange={(e) => set(e.target.value)}
        spellCheck={false}
        autoComplete="off"
        {...props}
        className={cn(FIELD, props.className)}
      />
    </label>
  );

  const hint = (
    <span className="text-sm leading-[1.45] text-text-muted">
      On the server, run <code className="font-mono">blotter-sync pair</code> for a code. It works
      once, for 10 minutes.
    </span>
  );

  return (
    <form
      className={cn(
        'flex flex-col gap-3',
        !bare && 'rounded-lg border border-border bg-bg px-3.5 py-3',
      )}
      onSubmit={(e) => {
        e.preventDefault();
        if (ready && !busy) void pair();
      }}
    >
      {bare ? (
        hint
      ) : (
        <div className="flex flex-col gap-[3px]">
          <span className="font-medium">{again ? 'Pair again' : 'Pair with a home server'}</span>
          {hint}
        </div>
      )}
      <div className="grid grid-cols-[2fr_1fr] gap-2">
        {field('Address', address, setAddress, {
          placeholder: '192.168.1.10 or https://sync.example.com',
          className: 'font-mono',
          autoFocus: !again,
        })}
        {field('Pairing code', code, (v) => setCode(v.toUpperCase()), {
          placeholder: 'ABCD-EFGH',
          className: 'font-mono tracking-[.06em]',
          autoFocus: again,
        })}
      </div>
      {field('This device’s name', name, setName, {
        placeholder: 'Desktop, Laptop…',
        maxLength: 64,
      })}
      <div className="flex items-center gap-3">
        <Button type="submit" variant="primary" disabled={!ready || busy}>
          {busy ? 'Pairing…' : again ? 'Pair again' : 'Pair'}
        </Button>
        {error && <span className="text-sm text-danger">{error}</span>}
      </div>
    </form>
  );
}

const CHOICES: readonly { value: JoinChoiceValue; name: string; desc: string }[] = [
  {
    value: 'server',
    name: 'Use the server’s planner',
    desc: 'Recommended for a new device. This device’s items, block-outs and settings are replaced by the server’s.',
  },
  {
    value: 'merge',
    name: 'Merge',
    desc: 'Keep both. Where both have the same tag, setting, keybinding or theme, the server’s wins, and public holidays aren’t doubled. Block-outs both made, like two “Work”, may need tidying.',
  },
];

/**
 * The server already has a planner and so does this device: use the
 * server's, or merge. Either way this device's planner is snapshotted
 * first. Cancelling unpairs.
 */
export function JoinChoice({
  address,
  onDone,
}: {
  address: string;
  /** `true` once joined, `false` when the pairing was cancelled. */
  onDone: (joined: boolean) => void;
}) {
  const [choice, setChoice] = useState<JoinChoiceValue>('server');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const act = async (command: 'sync_join' | 'sync_join_cancel') => {
    setBusy(true);
    setError(null);
    try {
      await invoke(command, command === 'sync_join' ? { choice } : undefined);
      onDone(command === 'sync_join');
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-3 rounded-lg border border-accent bg-bg px-3.5 py-3">
      <div className="flex flex-col gap-[3px]">
        <span className="font-medium">The server already has a planner</span>
        <span className="text-sm leading-[1.45] text-text-muted">
          Paired with <span className="font-mono">{address}</span>, and this device has a planner
          too. Whichever you pick, a snapshot of this device’s planner goes to Settings → Backups
          first.
        </span>
      </div>
      <div role="radiogroup" aria-label="How to join" className="grid grid-cols-2 gap-2">
        {CHOICES.map((c) => (
          <button
            key={c.value}
            type="button"
            role="radio"
            aria-checked={choice === c.value}
            onClick={() => setChoice(c.value)}
            className={cn(
              'flex cursor-pointer flex-col gap-[3px] rounded-md border p-3 text-left text-text',
              choice === c.value ? 'border-accent bg-accent-tint' : 'border-border bg-transparent',
            )}
          >
            <span className="font-medium">{c.name}</span>
            <span className="text-xs leading-[1.45] text-text-muted">{c.desc}</span>
          </button>
        ))}
      </div>
      <div className="flex items-center gap-3">
        <Button variant="primary" disabled={busy} onClick={() => void act('sync_join')}>
          {busy ? 'Joining…' : choice === 'server' ? 'Use the server’s planner' : 'Merge'}
        </Button>
        <Button variant="ghost" disabled={busy} onClick={() => void act('sync_join_cancel')}>
          Cancel pairing
        </Button>
        {error && <span className="text-sm text-danger">{error}</span>}
      </div>
    </div>
  );
}
