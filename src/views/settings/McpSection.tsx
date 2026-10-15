import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQuery } from '@tanstack/react-query';
import { Check, Copy, Terminal } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { isOpError } from '@/lib/api';
import { claudeCodeCommand, desktopConfig, type McpSetup } from '@/lib/mcpSetup';

function CopyBlock({ label, text }: { label: string; text: string }) {
  const [copied, setCopied] = useState(false);
  const copy = () =>
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    });
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center justify-between text-sm text-text-muted">
        {label}
        <Button size="sm" variant="ghost" onClick={copy} aria-label={`Copy ${label}`}>
          {copied ? <Check /> : <Copy />}
          {copied ? 'Copied' : 'Copy'}
        </Button>
      </div>
      <pre className="m-0 overflow-x-auto rounded-md border border-border bg-bg px-3 py-2 font-mono text-xs leading-normal text-text">
        {text}
      </pre>
    </div>
  );
}

/** Settings → AI providers: connecting Claude Desktop, Claude Code or another MCP client. */
export function McpSection() {
  const setup = useQuery({
    queryKey: ['mcp-setup'],
    queryFn: () => invoke<McpSetup>('mcp_setup'),
  });

  return (
    <section className="flex flex-col gap-3 rounded-lg border border-border p-4">
      <h2 className="m-0 flex items-center gap-2 text-md font-semibold">
        <Terminal size={15} />
        MCP server
      </h2>
      <p className="m-0 text-sm leading-normal text-text-muted">
        Lets Claude Desktop, Claude Code or any other MCP client read and change your planner
        through the same operations as the app. Changes are logged as <code>mcp</code>, can be
        undone with Ctrl+Z, and Blotter takes a snapshot before the first change of each session.
        The client starts the server itself; Blotter doesn&apos;t need to be open.
      </p>
      {setup.isError && (
        <p className="m-0 text-sm text-danger">
          {isOpError(setup.error) ? setup.error.message : String(setup.error)}
        </p>
      )}
      {setup.data && (
        <>
          <CopyBlock label="Claude Code" text={claudeCodeCommand(setup.data)} />
          <CopyBlock
            label="Claude Desktop (claude_desktop_config.json)"
            text={desktopConfig(setup.data)}
          />
        </>
      )}
    </section>
  );
}
