import { describe, expect, it } from 'vitest';
import { claudeCodeCommand, desktopConfig, type McpSetup } from './mcpSetup';

const win: McpSetup = {
  command: 'C:\\Program Files\\Blotter\\blotter.exe',
  args: ['mcp'],
  data_dir: null,
  data_dir_env: 'BLOTTER_DATA_DIR',
};

describe('desktopConfig', () => {
  it('names the program and the mcp subcommand', () => {
    expect(JSON.parse(desktopConfig(win))).toEqual({
      mcpServers: { blotter: { command: win.command, args: ['mcp'] } },
    });
  });

  it('passes a non-default data folder on', () => {
    const cfg = JSON.parse(desktopConfig({ ...win, data_dir: 'D:\\demo' }));
    expect(cfg.mcpServers.blotter.env).toEqual({ BLOTTER_DATA_DIR: 'D:\\demo' });
  });
});

describe('claudeCodeCommand', () => {
  it('quotes Windows paths as they are', () => {
    expect(claudeCodeCommand(win)).toBe(
      'claude mcp add --transport stdio --scope user blotter -- "C:\\Program Files\\Blotter\\blotter.exe" mcp',
    );
    expect(claudeCodeCommand({ ...win, data_dir: 'D:\\demo' })).toBe(
      'claude mcp add --transport stdio --scope user blotter --env BLOTTER_DATA_DIR="D:\\demo" -- "C:\\Program Files\\Blotter\\blotter.exe" mcp',
    );
  });

  it('escapes shell specials on Unix and puts the env before --', () => {
    const cmd = claudeCodeCommand({
      ...win,
      command: '/home/luke/my "apps"/blotter',
      data_dir: '/tmp/demo',
    });
    expect(cmd).toBe(
      'claude mcp add --transport stdio --scope user blotter --env BLOTTER_DATA_DIR="/tmp/demo" -- "/home/luke/my \\"apps\\"/blotter" mcp',
    );
  });
});
