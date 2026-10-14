import { Dialog } from 'radix-ui';
import { ACTION_GROUPS, type Action } from '@/actions/actions';
import { Kbd, KeyChips } from '@/components/ui/kbd';
import { useOverlay } from './overlay';

/** The `?` cheat sheet, generated from the action registry's bindings. */
export function ShortcutSheet({ actions }: { actions: readonly Action[] }) {
  const open = useOverlay((s) => s.overlay === 'shortcuts');
  const close = useOverlay((s) => s.close);
  const sections = ACTION_GROUPS.map((g) => ({
    name: g,
    rows: actions.filter((a) => a.group === g && a.binding),
  })).filter((s) => s.rows.length > 0);

  return (
    <Dialog.Root open={open} onOpenChange={(o) => !o && close()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-[color-mix(in_srgb,var(--color-bg)_62%,transparent)]" />
        <Dialog.Content
          aria-describedby={undefined}
          // Focus the panel itself rather than the Esc button, which would
          // otherwise open with a focus ring on it.
          tabIndex={-1}
          onOpenAutoFocus={(e) => {
            e.preventDefault();
            (e.currentTarget as HTMLElement | null)?.focus();
          }}
          className="fixed top-[110px] left-1/2 z-41 flex w-[880px] max-w-[calc(100vw-32px)] -translate-x-1/2 flex-col gap-4 rounded-lg bg-surface-raised px-6 pt-5 pb-[22px] shadow-overlay focus:outline-none"
        >
          <div className="flex items-center gap-2.5">
            <Dialog.Title className="m-0 font-heading text-xl font-heading-weight">
              Keyboard shortcuts
            </Dialog.Title>
            <span className="flex-1" />
            <Dialog.Close className="cursor-pointer" aria-label="Close">
              <Kbd>Esc</Kbd>
            </Dialog.Close>
          </div>
          <div className="grid grid-cols-3 gap-x-7 gap-y-5">
            {sections.map((s) => (
              <section key={s.name} className="flex flex-col gap-0.5">
                <h3 className="m-0 pb-1.5 text-xs font-normal tracking-[.06em] text-text-muted uppercase">
                  {s.name}
                </h3>
                {s.rows.map((a) => (
                  <div key={a.id} className="flex h-7 items-center gap-2 border-b border-border">
                    <span className="flex-1">{a.label}</span>
                    {a.binding && <KeyChips binding={a.binding} />}
                  </div>
                ))}
              </section>
            ))}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
