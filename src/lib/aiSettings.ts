/** Which AI the chat uses: Claude Code (default, D29) or the default API provider. */
export type Route = 'claude_code' | 'api';

/** The app's AI settings (`get_ai_settings` / `set_ai_settings`). */
export interface AiSettings {
  route: Route;
  claude_code_model: string;
  /** Empty: Blotter finds `claude` itself. */
  claude_code_path: string;
}

export const AI_SETTINGS_KEY = ['ai-settings'];
