/** How an MCP client launches Blotter's server (from the `mcp_setup` command). */
export interface McpSetup {
  command: string;
  args: string[];
  /** Set when Blotter runs on a non-default database. */
  data_dir: string | null;
  data_dir_env: string;
}

/** Double-quotes for a POSIX shell, escaping what is special inside them. */
const quote = (s: string) => `"${s.replace(/(["\\$`])/g, '\\$1')}"`;

/** The `mcpServers` entry for Claude Desktop's config (and other JSON-configured clients). */
export function desktopConfig(setup: McpSetup): string {
  const server: Record<string, unknown> = { command: setup.command, args: setup.args };
  if (setup.data_dir) server.env = { [setup.data_dir_env]: setup.data_dir };
  return JSON.stringify({ mcpServers: { blotter: server } }, null, 2);
}

/** A `claude mcp add` line that registers Blotter for every Claude Code project. */
export function claudeCodeCommand(setup: McpSetup): string {
  // Windows paths keep their backslashes, which aren't escapes there.
  const win = /^[A-Za-z]:\\/.test(setup.command);
  const q = (s: string) => (win ? `"${s}"` : quote(s));
  const env = setup.data_dir ? ` --env ${setup.data_dir_env}=${q(setup.data_dir)}` : '';
  return `claude mcp add --transport stdio --scope user blotter${env} -- ${q(setup.command)} ${setup.args.join(' ')}`;
}
