CREATE SCHEMA advent_practice;
CREATE TABLE advent_practice.attempts (LIKE public.attempts INCLUDING ALL);
ALTER TABLE advent_practice.attempts ADD FOREIGN KEY (uid) REFERENCES public.users(id);
CREATE TABLE advent_practice.events (LIKE public.events INCLUDING ALL);
ALTER TABLE advent_practice.events ADD FOREIGN KEY (attempt) REFERENCES advent_practice.attempts(id) ON DELETE CASCADE;
