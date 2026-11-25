CREATE TABLE season (id bool PRIMARY KEY CHECK(id), year int NOT NULL, seed text NOT NULL, commitment text NOT NULL, shift interval NOT NULL DEFAULT '0', snapshot jsonb, digest text, winners jsonb);
CREATE TABLE users(id bigint PRIMARY KEY, alias uuid NOT NULL UNIQUE DEFAULT gen_random_uuid(), nick text NOT NULL, location text NOT NULL DEFAULT '', avatar text NOT NULL DEFAULT '⚽', notify bool NOT NULL DEFAULT false, sound bool NOT NULL DEFAULT true, motion bool NOT NULL DEFAULT true, haptic bool NOT NULL DEFAULT true);
CREATE TABLE sessions(token text PRIMARY KEY, uid bigint NOT NULL REFERENCES users, expires timestamptz NOT NULL);
CREATE INDEX sessions_expiry ON sessions(expires);
CREATE TABLE doors(uid bigint REFERENCES users, day int CHECK(day BETWEEN 1 AND 20), PRIMARY KEY(uid,day));
CREATE TABLE spins(uid bigint REFERENCES users,day int CHECK(day BETWEEN 1 AND 20),reward int NOT NULL CHECK(reward BETWEEN 1 AND 5),PRIMARY KEY(uid,day));
CREATE TABLE attempts(id uuid PRIMARY KEY DEFAULT gen_random_uuid(),uid bigint NOT NULL REFERENCES users,game int NOT NULL CHECK(game BETWEEN 1 AND 10),slot int NOT NULL CHECK(slot BETWEEN 1 AND 3),deadline timestamptz NOT NULL,issued timestamptz NOT NULL,pressed timestamptz,scene jsonb NOT NULL,step int NOT NULL DEFAULT 0,hits int NOT NULL DEFAULT 0,score int NOT NULL DEFAULT 0,combo int NOT NULL DEFAULT 0,done bool NOT NULL DEFAULT false,flagged bool NOT NULL DEFAULT false, UNIQUE(uid,game,slot));
CREATE TABLE events(attempt uuid REFERENCES attempts,step int,input jsonb NOT NULL,ms int NOT NULL,points int NOT NULL,PRIMARY KEY(attempt,step));
CREATE TABLE tickets(id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,uid bigint NOT NULL REFERENCES users,day int NOT NULL CHECK(day BETWEEN 1 AND 20),source text NOT NULL CHECK(source IN ('game','wheel')),part int NOT NULL CHECK(part BETWEEN 1 AND 5),place int,earned timestamptz NOT NULL,valid bool NOT NULL DEFAULT true,UNIQUE(uid,day,source,part));
CREATE INDEX tickets_owner ON tickets(uid);
CREATE VIEW rankings AS SELECT *,rank() OVER(PARTITION BY game ORDER BY score DESC,hits DESC)::int place FROM (SELECT DISTINCT ON(game,uid) game,uid,score,hits FROM attempts WHERE done AND NOT flagged AND step>0 ORDER BY game,uid,score DESC,hits DESC) b;
CREATE TABLE closed(game int PRIMARY KEY);
CREATE TABLE final_ranks(game int REFERENCES closed,uid bigint REFERENCES users,score int,hits int,place int,PRIMARY KEY(game,uid));
CREATE TABLE prizes(id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,title text NOT NULL, request uuid UNIQUE NOT NULL);
CREATE TABLE jobs(uid bigint REFERENCES users,kind text,body text NOT NULL,status text NOT NULL DEFAULT 'queued',PRIMARY KEY(uid,kind));
CREATE TABLE audit(id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY, actor bigint REFERENCES users,action text NOT NULL,detail jsonb NOT NULL,created timestamptz NOT NULL DEFAULT now());

CREATE FUNCTION event_time() RETURNS timestamptz LANGUAGE sql STABLE AS $$ SELECT statement_timestamp()+shift FROM season $$;
CREATE FUNCTION event_day(t timestamptz) RETURNS int LANGUAGE sql STABLE AS $$SELECT (t AT TIME ZONE 'Europe/Minsk')::date-make_date(year,12,10) FROM season$$;
CREATE FUNCTION wheel_day(t timestamptz) RETURNS int LANGUAGE sql STABLE AS $$SELECT CASE WHEN event_day(t) BETWEEN 1 AND 20 AND (t AT TIME ZONE 'Europe/Minsk')::time>='09:00' THEN event_day(t) END$$;
CREATE FUNCTION game_at(t timestamptz) RETURNS int LANGUAGE sql STABLE AS $$SELECT CASE WHEN event_day(t) BETWEEN 1 AND 20 AND (event_day(t)%2=0 OR wheel_day(t) IS NOT NULL) THEN (event_day(t)+1)/2 END$$;
CREATE FUNCTION closes(g int) RETURNS timestamptz LANGUAGE sql STABLE AS $$SELECT make_date(year,12,11+2*g)::timestamp AT TIME ZONE 'Europe/Minsk' FROM season$$;
CREATE FUNCTION award(p int) RETURNS int LANGUAGE sql IMMUTABLE AS $$SELECT CASE WHEN p<=5 THEN 5 WHEN p<=10 THEN 4 WHEN p<=20 THEN 3 WHEN p<=30 THEN 2 ELSE 1 END$$;
CREATE FUNCTION settle(t timestamptz) RETURNS void LANGUAGE plpgsql AS $$
DECLARE g int; inserted int;
BEGIN
 UPDATE attempts SET done=true WHERE NOT done AND deadline<=t;
 FOR g IN 1..10 LOOP
  EXIT WHEN t<closes(g);
  INSERT INTO closed VALUES(g) ON CONFLICT DO NOTHING;
  GET DIAGNOSTICS inserted=ROW_COUNT;
  IF inserted=1 THEN
   INSERT INTO final_ranks SELECT * FROM rankings WHERE game=g;
   INSERT INTO tickets(uid,day,source,part,place,earned) SELECT uid,g*2,'game',part,place,t FROM final_ranks CROSS JOIN LATERAL generate_series(1,award(place)) part WHERE game=g ON CONFLICT DO NOTHING;
   INSERT INTO jobs(uid,kind,body) SELECT uid,'rank-'||g,format('Итоги игры %s. Место %s, билетов начислено: %s.',g,place,award(place)) FROM final_ranks JOIN users ON id=uid WHERE game=g AND notify ON CONFLICT DO NOTHING;
  END IF;
 END LOOP;
END$$;
CREATE VIEW proof AS SELECT jsonb_build_object('commitment',commitment,'seed',CASE WHEN winners IS NOT NULL THEN seed END,'snapshot',snapshot,'digest',digest,'winners',winners,'prizes',coalesce((SELECT jsonb_agg(title ORDER BY id) FROM prizes),'[]')) data FROM season;
