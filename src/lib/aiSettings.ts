/** Which AI the chat uses: Claude Code (default, D29), Codex, or the default API provider. */
export type Route = 'claude_code' | 'codex' | 'api';

/** The app's AI settings (`get_ai_settings` / `set_ai_settings`). */
/** Whether AI and MCP changes wait for approval (remembered across restarts). */
export type AiMode = 'propose' | 'auto';

export interface AiSettings {
  route: Route;
  mode: AiMode;
  claude_code_model: string;
  /** Empty: Blotter finds `claude` itself. */
  claude_code_path: string;
  /** Empty: Codex's own default model. */
  codex_model: string;
  /** Empty: Blotter finds `codex` itself. */
  codex_path: string;
}

export const AI_SETTINGS_KEY = ['ai-settings'];
