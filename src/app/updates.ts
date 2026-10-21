import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { isOpError } from '@/lib/api';
import { useNotice } from './noticeStore';

/** A newer release, from the backend's update check. */
export interface UpdateInfo {
  version: string;
  current: string;
  notes_url: string;
  /** Blotter can install it itself; otherwise `how` says what to do. */
  can_install: boolean;
  how: string | null;
}

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));
const notice = () => useNotice.getState();

/** Downloads and installs it; Blotter restarts into the new version. */
export function installUpdate(info: UpdateInfo): void {
  notice().show({ text: `Downloading Blotter ${info.version}… Blotter restarts when it's in.` });
  invoke('install_update').catch((e: unknown) =>
    notice().show({ text: errorText(e), tone: 'error' }),
  );
}

/** "Blotter 1.2.1 is out", with Restart to update or the release notes. */
export function announceUpdate(info: UpdateInfo): void {
  notice().show({
    text: info.can_install
      ? `Blotter ${info.version} is out.`
      : `Blotter ${info.version} is out. ${info.how ?? ''}`,
    action: info.can_install
      ? { label: 'Restart to update', run: () => installUpdate(info) }
      : {
          label: 'Release notes',
          run: () => void invoke('open_release_notes', { version: info.version }),
        },
  });
}

/** Looks now; says so when it's already up to date. */
export async function checkForUpdates(): Promise<UpdateInfo | null> {
  try {
    const info = await invoke<UpdateInfo | null>('check_for_update');
    if (info) announceUpdate(info);
    else notice().show({ text: 'Blotter is up to date.' });
    return info;
  } catch (e) {
    notice().show({ text: errorText(e), tone: 'error' });
    return null;
  }
}

/** Mounted once in the main window: shows what the daily check finds. */
export function useUpdateNotices(): void {
  useEffect(() => {
    invoke<UpdateInfo | null>('available_update')
      .then((info) => info && announceUpdate(info))
      .catch(() => undefined);
    const stop = listen<UpdateInfo>('planner:update', (e) => announceUpdate(e.payload));
    return () => void stop.then((unlisten) => unlisten());
  }, []);
}
