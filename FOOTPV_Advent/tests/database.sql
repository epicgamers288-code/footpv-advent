-- Run against a disposable migrated rehearsal database. All changes roll back.
BEGIN;
DO $$
DECLARE d int; t timestamptz; i int; n int;
BEGIN
 IF current_database() NOT LIKE '%\_test' THEN RAISE EXCEPTION 'Use a rehearsal database';END IF;
 FOR d IN 1..20 LOOP
  t=(make_date(2026,12,d+10)+time '09:00') AT TIME ZONE 'Europe/Minsk';
  IF wheel_day(t) IS DISTINCT FROM d OR wheel_day(t-interval '1 second') IS NOT NULL OR game_at(t) IS DISTINCT FROM (d+1)/2 THEN RAISE EXCEPTION 'Opening %',d;END IF;
 END LOOP;
 IF game_at(closes(10)) IS NOT NULL THEN RAISE EXCEPTION 'Final deadline';END IF;
 INSERT INTO users(id,nick) VALUES (-1,'А'),(-2,'Б'),(-3,'В');
 INSERT INTO attempts(uid,game,slot,deadline,issued,scene,score,step,hits,done)
 SELECT uid,g,1,closes(g),closes(g)-interval '30 seconds','{}',100,2,CASE WHEN uid=-3 THEN 1 ELSE 2 END,true FROM generate_series(1,10)g CROSS JOIN unnest(ARRAY[-1,-2,-3])uid;
 INSERT INTO tickets(uid,day,source,part,earned) SELECT -1,dg,'wheel',part,now() FROM generate_series(1,20)dg CROSS JOIN generate_series(1,5)part;
 PERFORM settle(closes(10));PERFORM settle(closes(10));
 SELECT count(*) INTO n FROM tickets WHERE uid=-1 AND valid;IF n<>150 THEN RAISE EXCEPTION 'Expected 150, got %',n;END IF;
 IF EXISTS(SELECT FROM final_ranks WHERE uid IN (-1,-2) AND place<>1) OR EXISTS(SELECT FROM final_ranks WHERE uid=-3 AND place<>3) THEN RAISE EXCEPTION 'Tie handling';END IF;
 IF ARRAY[award(1),award(5),award(6),award(10),award(11),award(20),award(21),award(30),award(31)]<>ARRAY[5,5,4,4,3,3,2,2,1] THEN RAISE EXCEPTION 'Prize brackets';END IF;
END$$;
ROLLBACK;
