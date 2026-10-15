import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { QueryClientProvider } from '@tanstack/react-query';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { AppShell } from './app/AppShell';
import { CaptureApp } from './app/CaptureApp';
import { ChatApp } from './app/ChatApp';
import { ErrorBoundary } from './app/ErrorBoundary';
import { TooltipProvider } from './components/ui/tooltip';
import { queryClient } from './lib/queries';
import { bootTheme } from './theme/theme';
import './styles.css';

// Apply the last-used theme before the first paint so there is no flash.
bootTheme();

const root = document.getElementById('root');
if (!root) throw new Error('Missing #root element');

// Every window loads this page; the quick-capture popup and the chat
// pop-out render only their panel.
const label = getCurrentWindow().label;

createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <ErrorBoundary what="Blotter" reload>
          {label === 'capture' ? <CaptureApp /> : label === 'chat' ? <ChatApp /> : <AppShell />}
        </ErrorBoundary>
      </TooltipProvider>
    </QueryClientProvider>
  </StrictMode>,
);
