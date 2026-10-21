-- Holidays no longer lift block-outs that warn about fixed items (Sleep):
-- drop the lifts existing holidays made on those.
DELETE FROM blockout_exceptions
WHERE holiday_id IS NOT NULL
  AND blockout_id IN (SELECT id FROM blockouts WHERE permits_fixed = 0);
