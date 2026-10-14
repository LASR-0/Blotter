import { useState } from 'react';
import { Dialog } from 'radix-ui';
import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';
import { isOpError } from '@/lib/api';
import { useOverlay } from './overlay';

/** A small yes/no dialog. The confirm button has focus, so Enter confirms. */
export function ConfirmDialog() {
  const request = useOverlay((s) => (s.overlay === 'confirm' ? s.confirm : null));
  const close = useOverlay((s) => s.close);
  const [error, setError] = useState<string | null>(null);

  const confirm = async () => {
    if (!request) return;
    try {
      await request.onConfirm();
      setError(null);
      close();
    } catch (e) {
      setError(isOpError(e) ? e.message : String(e));
    }
  };

  return (
    <Dialog.Root
      open={request !== null}
      onOpenChange={(o) => {
        if (!o) {
          setError(null);
          close();
        }
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-[color-mix(in_srgb,var(--color-bg)_62%,transparent)]" />
        <Dialog.Content
          aria-describedby={request?.body ? 'confirm-body' : undefined}
          className="fixed top-[140px] left-1/2 z-41 flex w-[380px] max-w-[calc(100vw-32px)] -translate-x-1/2 flex-col gap-3 rounded-lg bg-surface-raised p-[18px] text-base text-text shadow-overlay focus:outline-none"
        >
          <Dialog.Title className="m-0 text-md font-semibold">{request?.title}</Dialog.Title>
          {request?.body && (
            <p id="confirm-body" className="m-0 leading-normal text-text-muted">
              {request.body}
            </p>
          )}
          {error && <p className="m-0 font-mono text-xs text-danger">{error}</p>}
          <div className="flex items-center justify-end gap-2 pt-1">
            <Dialog.Close asChild>
              <Button>
                Cancel <Kbd>Esc</Kbd>
              </Button>
            </Dialog.Close>
            <Button
              autoFocus
              variant={request?.danger ? 'danger' : 'primary'}
              onClick={() => void confirm()}
            >
              {request?.confirmLabel}
              <Kbd onAccent>↵</Kbd>
            </Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
