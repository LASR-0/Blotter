-- A thread run through Claude Code remembers its session, so the next
-- message resumes it (`claude --resume`) instead of starting over.
ALTER TABLE chat_threads ADD COLUMN external_session TEXT;
