-- The entity ids a proposed operation's rehearsal created, handed out again
-- when it's accepted, so later proposals that refer to them still work.
ALTER TABLE proposal_ops ADD COLUMN ids_json TEXT;
-- Closed to new operations (the chat turn that made it has ended); the
-- next proposal from the same AI starts a new card.
ALTER TABLE proposals ADD COLUMN sealed INTEGER NOT NULL DEFAULT 0 CHECK (sealed IN (0, 1));
