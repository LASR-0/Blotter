import { Component, type ErrorInfo, type ReactNode } from 'react';
import { ErrorPanel } from '@/components/EmptyState';

interface Props {
  /** What broke, e.g. "This view" or "Blotter". */
  what: string;
  /** Offer reloading the window, for the outermost boundary. */
  reload?: boolean;
  children: ReactNode;
}

interface State {
  error: Error | null;
}

/**
 * Shows what went wrong instead of a blank window when rendering throws.
 * The view-level boundary is keyed by view, so navigating away resets it;
 * "Try again" re-renders in place. Your data is in the database either way.
 */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: unknown): State {
    return { error: error instanceof Error ? error : new Error(String(error)) };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error(`${this.props.what} crashed`, error, info.componentStack);
  }

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;
    return (
      <div className="flex h-full items-start justify-center overflow-auto p-8">
        <div className="flex w-full max-w-[560px] flex-col gap-3">
          <ErrorPanel
            title={`${this.props.what} hit an error and couldn't be shown`}
            message={error.message}
            onRetry={() => this.setState({ error: null })}
          />
          <p className="m-0 text-sm leading-normal text-text-muted">
            Nothing was lost: your planner data is saved as you go.
            {this.props.reload
              ? ' Reloading the window usually clears it.'
              : ' Switching to another view also resets it.'}
          </p>
          {this.props.reload && (
            <button
              type="button"
              onClick={() => window.location.reload()}
              className="h-[30px] cursor-pointer self-start rounded-md border border-border-strong px-3 text-base hover:bg-hover"
            >
              Reload window
            </button>
          )}
        </div>
      </div>
    );
  }
}
