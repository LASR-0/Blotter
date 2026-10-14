import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { QueryClientProvider } from '@tanstack/react-query';
import { AppShell } from './app/AppShell';
import { TooltipProvider } from './components/ui/tooltip';
import { queryClient } from './lib/queries';
import { bootTheme } from './theme/theme';
import './styles.css';

// Apply the last-used theme before the first paint so there is no flash.
bootTheme();

const root = document.getElementById('root');
if (!root) throw new Error('Missing #root element');

createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <AppShell />
      </TooltipProvider>
    </QueryClientProvider>
  </StrictMode>,
);
