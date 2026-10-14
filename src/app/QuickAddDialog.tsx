import { Dialog } from 'radix-ui';
import { QuickAddPanel } from '@/components/QuickAddPanel';
import { useFocus } from './focus';
import { useOverlay } from './overlay';

/** The `n` quick-add bar, shown as an overlay (design V8). */
export function QuickAddDialog() {
  const open = useOverlay((s) => s.overlay === 'quickadd');
  const close = useOverlay((s) => s.close);
  return (
    <Dialog.Root open={open} onOpenChange={(o) => !o && close()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-[color-mix(in_srgb,var(--color-bg)_62%,transparent)]" />
        <Dialog.Content
          aria-describedby={undefined}
          // The panel handles Esc itself (so it can be reused in the capture window).
          onEscapeKeyDown={(e) => e.preventDefault()}
          className="fixed top-[110px] left-1/2 z-41 w-[600px] max-w-[calc(100vw-32px)] -translate-x-1/2 focus:outline-none"
        >
          <Dialog.Title className="sr-only">Quick add</Dialog.Title>
          {open && (
            <QuickAddPanel
              onClose={close}
              onCreated={(item, details) => {
                useFocus.getState().setItem(item.id);
                if (details) useOverlay.getState().openItem({ id: item.id });
                else close();
              }}
            />
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
