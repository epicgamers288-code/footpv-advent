ALTER TABLE tickets DROP CONSTRAINT tickets_source_check;
ALTER TABLE tickets ADD CHECK (source IN ('game','wheel') OR (source='runner' AND day=2 AND part BETWEEN 1 AND 3));
ALTER TABLE public.attempts ADD COLUMN runner_pickups int NOT NULL DEFAULT 0;
ALTER TABLE advent_practice.attempts ADD COLUMN runner_pickups int NOT NULL DEFAULT 0;
