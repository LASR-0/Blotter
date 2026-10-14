import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { TooltipProvider } from './components/ui/tooltip';
import { bootTheme } from './theme/theme';
import './styles.css';

// Apply the last-used theme before the first paint so there is no flash.
bootTheme();

const root = document.getElementById('root');
if (!root) throw new Error('Missing #root element');

createRoot(root).render(
  <StrictMode>
    <TooltipProvider>
      <App />
    </TooltipProvider>
  </StrictMode>,
);
