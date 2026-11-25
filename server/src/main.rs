mod assets;
mod game;
mod staff;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use hmac::{Hmac, Mac};
use rand::{Rng, rngs::OsRng};
use serde_json::{Value as V, json};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use std::{
    collections::{BTreeMap, HashMap},
    env,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
type Tx<'a> = Transaction<'a, Postgres>;
type R<T> = Result<T, Fault>;
#[derive(Debug)]
struct Fault(StatusCode, String);
impl axum::response::IntoResponse for Fault {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
impl From<sqlx::Error> for Fault {
    fn from(e: sqlx::Error) -> Self {
        eprintln!("PostgreSQL: {e}");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Ошибка сохранения. Обнови экран.".into(),
        )
    }
}
fn bad(msg: &str) -> Fault {
    Fault(StatusCode::BAD_REQUEST, msg.into())
}
fn deny() -> Fault {
    Fault(StatusCode::FORBIDDEN, "Доступ закрыт".into())
}
fn hash(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}
fn random() -> String {
    hex::encode(OsRng.r#gen::<[u8; 32]>())
}
#[derive(Clone)]
struct App {
    db: PgPool,
    bot: String,
    admins: Vec<i64>,
    limits: Arc<tokio::sync::Mutex<HashMap<i64, (Instant, u32)>>>,
}
async fn limit(a: &App, id: i64, max: u32) -> R<()> {
    let mut m = a.limits.lock().await;
    m.retain(|_, e| e.0.elapsed().as_secs() < 60);
    let e = m.entry(id).or_insert((Instant::now(), 0));
    if e.1 >= max {
        return Err(Fault(
            StatusCode::TOO_MANY_REQUESTS,
            "Подожди минуту".into(),
        ));
    }
    e.1 += 1;
    Ok(())
}
fn verified(raw: &str, bot: &str) -> R<i64> {
    let mut vals = BTreeMap::new();
    for (k, v) in url::form_urlencoded::parse(raw.as_bytes()) {
        if vals.insert(k.into_owned(), v.into_owned()).is_some() {
            return Err(deny());
        }
    }
    let sig = hex::decode(vals.remove("hash").ok_or_else(deny)?).map_err(|_| deny())?;
    let payload = vals
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");
    let key = Hmac::<Sha256>::new_from_slice(b"WebAppData")
        .unwrap()
        .chain_update(bot)
        .finalize()
        .into_bytes();
    Hmac::<Sha256>::new_from_slice(&key)
        .unwrap()
        .chain_update(payload)
        .verify_slice(&sig)
        .map_err(|_| deny())?;
    let stamp = vals
        .get("auth_date")
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or_else(deny)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if now.checked_sub(stamp).is_none_or(|n| n > 600) {
        return Err(deny());
    }
    let user: V = serde_json::from_str(vals.get("user").ok_or_else(deny)?).map_err(|_| deny())?;
    user["id"].as_i64().filter(|i| *i > 0).ok_or_else(deny)
}
async fn login(State(a): State<App>, Json(v): Json<V>) -> R<Json<V>> {
    limit(&a, 0, 1000).await?;
    let id = if cfg!(feature = "rehearsal") {
        v["testUser"]
            .as_i64()
            .filter(|x| (1..=99999).contains(x))
            .ok_or_else(deny)?
    } else {
        verified(v["initData"].as_str().unwrap_or(""), &a.bot)?
    };
    let token = random();
    let mut tx = a.db.begin().await?;
    sqlx::query("insert into users(id,nick) values($1,$2) on conflict do nothing")
        .bind(id)
        .bind(format!("Игрок {}", &hash(&id.to_string())[..6]))
        .execute(&mut *tx)
        .await?;
    sqlx::query("insert into sessions values($1,$2,now()+interval '6 hours')")
        .bind(hash(&token))
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({"token":token})))
}
async fn public_proof(State(a): State<App>) -> R<Json<V>> {
    Ok(Json(
        sqlx::query_scalar("select data from proof")
            .fetch_one(&a.db)
            .await?,
    ))
}
const ROUND: &str = "select (to_jsonb(a)-'uid'-'flagged'-'pressed')||jsonb_build_object('scene',scene-'answer','now',event_time()) from attempts a where id::text=$1 and uid=$2";
async fn round(tx: &mut Tx<'_>, id: &str, uid: i64) -> R<V> {
    sqlx::query_scalar(ROUND)
        .bind(id)
        .bind(uid)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(deny)
}
async fn rpc(
    State(a): State<App>,
    Path(op): Path<String>,
    h: HeaderMap,
    Json(v): Json<V>,
) -> R<Json<V>> {
    let tok = h
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(deny)?;
    let uid: i64 = sqlx::query_scalar("select uid from sessions where token=$1 and expires>now()")
        .bind(hash(tok))
        .fetch_optional(&a.db)
        .await?
        .ok_or_else(deny)?;
    limit(&a, uid, 300).await?;
    let mut tx = a.db.begin().await?;
    let exclusive = matches!(
        op.as_str(),
        "freeze" | "draw" | "review" | "clock" | "prize"
    );
    sqlx::query(if exclusive {
        "select pg_advisory_xact_lock(817402)"
    } else {
        "select pg_advisory_xact_lock_shared(817402)"
    })
    .execute(&mut *tx)
    .await?;
    sqlx::query("select id from users where id=$1 for update")
        .bind(uid)
        .fetch_one(&mut *tx)
        .await?;
    // Practice uses the same scoring engine with transaction-local isolated tables.
    let practice = matches!(op.as_str(), "practice-start" | "practice-action" | "practice-charge" | "practice-finish");
    if practice {
        if !a.admins.contains(&uid) { return Err(deny()); }
        sqlx::query("set local search_path to advent_practice, public").execute(&mut *tx).await?;
    }
    let op = op.strip_prefix("practice-").filter(|_|practice).unwrap_or(&op);
    let date = sqlx::query(
        "select event_day(event_time()) d,wheel_day(event_time()) w,game_at(event_time()) g",
    )
    .fetch_one(&mut *tx)
    .await?;
    let day: i32 = date.get("d");
    let wheel: Option<i32> = date.get("w");
    let game: Option<i32> = date.get("g");
    let out = match op {
        "state" => {
            let mut s: V = sqlx::query_scalar(include_str!("state.sql"))
                .bind(uid)
                .fetch_one(&mut *tx)
                .await?;
            let unlocked = if day < 1 {
                0
            } else if day > 20 {
                10
            } else {
                game.unwrap_or(day / 2)
            };
            s["games"] = json!(&game::GAMES[..unlocked as usize]);
            s["admin"] = json!(a.admins.contains(&uid));
            if a.admins.contains(&uid) { s["practiceGames"]=json!(game::GAMES); }
            s["rehearsal"] = json!(cfg!(feature = "rehearsal"));
            s
        }
        "open" | "spin" => {
            let d = wheel
                .filter(|d| v["day"] == *d)
                .ok_or_else(|| bad("Дверца открывается в 09:00 по Минску"))?;
            sqlx::query("insert into doors values($1,$2) on conflict do nothing")
                .bind(uid)
                .bind(d)
                .execute(&mut *tx)
                .await?;
            if op == "open" {
                json!(true)
            } else {
                let n:i32=sqlx::query_scalar("insert into spins values($1,$2,$3) on conflict(uid,day) do update set reward=spins.reward returning reward").bind(uid).bind(d).bind(game::reward(OsRng.gen_range(0..100))).fetch_one(&mut *tx).await?;
                sqlx::query("insert into tickets(uid,day,source,part,earned) select $1,$2,'wheel',i,event_time() from generate_series(1,$3)i on conflict do nothing").bind(uid).bind(d).bind(n).execute(&mut *tx).await?;
                json!({"reward":n})
            }
        }
        "start" if practice => {
            let g=v["game"].as_i64().filter(|n|(1..=10).contains(n)).ok_or_else(||bad("Выберите игру"))? as i32;
            // Each administrator may restart any drill. Keep at most one run per game.
            sqlx::query("delete from advent_practice.attempts where uid=$1 and game=$2").bind(uid).bind(g).execute(&mut *tx).await?;
            let id:String=sqlx::query_scalar("insert into advent_practice.attempts(uid,game,slot,scene,issued,deadline) values($1,$2,1,$3,event_time(),event_time()+$4*interval '1 second') returning id::text")
                .bind(uid).bind(g).bind(game::scene(g,0)).bind(f64::from(game::GAMES[(g-1)as usize].2)).fetch_one(&mut *tx).await?;
            round(&mut tx,&id,uid).await?
        }
        "start" => {
            let g = game
                .filter(|g| v["game"] == *g)
                .ok_or_else(|| bad("Игра закрыта"))?;
            let active:Option<String>=sqlx::query_scalar("select id::text from attempts where uid=$1 and game=$2 and deadline>event_time() and not done").bind(uid).bind(g).fetch_optional(&mut *tx).await?;
            let id = if let Some(id) = active {
                id
            } else {
                let count: i64 =
                    sqlx::query_scalar("select count(*) from attempts where uid=$1 and game=$2")
                        .bind(uid)
                        .bind(g)
                        .fetch_one(&mut *tx)
                        .await?;
                if count >= 3 {
                    return Err(bad("Все три попытки сыграны"));
                }
                sqlx::query_scalar("insert into attempts(uid,game,slot,scene,issued,deadline) select $1,$2,$3,$4,event_time(),event_time()+$5*interval '1 second' where event_time()+$5*interval '1 second'<=closes($2) returning id::text").bind(uid).bind(g).bind((count+1)as i32).bind(game::scene(g,0)).bind(f64::from(game::GAMES[(g-1)as usize].2)).fetch_optional(&mut *tx).await?.ok_or_else(||bad("До финального свистка недостаточно времени"))?
            };
            round(&mut tx, &id, uid).await?
        }
        "action" | "charge" | "finish" => {
            let id = v["id"].as_str().unwrap_or("");
            let r=sqlx::query("select *,event_time()>=deadline expired,(extract(epoch from(event_time()-issued))*1000)::int ms,coalesce((extract(epoch from(event_time()-pressed))*1000)::int,0) power from attempts where id::text=$1 and uid=$2").bind(id).bind(uid).fetch_optional(&mut *tx).await?.ok_or_else(deny)?;
            let step: i32 = r.get("step");
            let c: V = r.get("scene");
            let g: i32 = r.get("game");
            let mut points = None;
            if r.get::<bool, _>("expired") || r.get::<bool, _>("done") {
                sqlx::query("update attempts set done=true where id::text=$1")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            } else if op == "finish" {
                return Err(bad("Время ещё идёт"));
            } else if v["step"] == step {
                if op == "charge" {
                    if c["kind"] != 8 {
                        return Err(bad("Замах недоступен"));
                    }
                    sqlx::query("update attempts set pressed=coalesce(pressed,event_time()) where id::text=$1").bind(id).execute(&mut *tx).await?;
                } else {
                    let ms: i32 = r.get("ms");
                    if ms < 160 {
                        return Err(bad("Слишком рано"));
                    }
                    let runner = g == 1 && c["pace"].is_number();
                    let mut pickup = 0;
                    let base = if runner {
                        let pace = c["pace"].as_i64().unwrap() as i32;
                        if ms < pace - 100 { return Err(bad("Препятствие ещё впереди")); }
                        let lane = v["lane"].as_i64().filter(|v|(0..3).contains(v)).ok_or_else(||bad("Неизвестная дорожка"))?;
                        let clear = Some(lane) == c["lane"].as_i64();
                        let dodge = (c["obstacle"] == "hurdle" && v["move"] == "jump") ||
                                    (c["obstacle"] == "arch" && v["move"] == "slide");
                        if ms <= pace + 1400 && (clear || dodge) {
                            if clear { pickup = c["part"].as_i64().unwrap_or(0) as i32; }
                            100
                        } else { 0 }
                    } else { game::score(&c, &v, ms, r.get("power")) };
                    let combo = if base > 0 {
                        r.get::<i32, _>("combo") + 1
                    } else {
                        0
                    };
                    let n = base
                        + if matches!(g, 6 | 9) {
                            3 * combo.min(10)
                        } else {
                            0
                        };
                    sqlx::query("insert into events values($1::text::uuid,$2,$3,$4,$5)")
                        .bind(id)
                        .bind(step)
                        .bind(&v)
                        .bind(ms)
                        .bind(n)
                        .execute(&mut *tx)
                        .await?;
                    let mut next_scene = game::scene(g, step + 1);
                    if g == 1 && !runner {
                        // Finish an attempt started by the previous client with its original rules.
                        next_scene.as_object_mut().unwrap().remove("pace");
                        next_scene["tip"] = c["tip"].clone();
                    }
                    sqlx::query("update attempts set step=step+1,score=score+$2,hits=hits+$3,combo=$4,scene=$5,issued=event_time(),pressed=null,flagged=flagged or ($6=19 and (select stddev_pop(ms)<10 and avg(ms)<450 from events where attempt=attempts.id)) where id::text=$1").bind(id).bind(n).bind(i32::from(n>0)).bind(combo).bind(next_scene).bind(step).execute(&mut *tx).await?;
                    if runner {
                        sqlx::query("update attempts set done=($2>=14), runner_pickups=runner_pickups+$3 where id::text=$1")
                            .bind(id).bind(step).bind(i32::from(pickup > 0)).execute(&mut *tx).await?;
                        if pickup > 0 && !practice {
                            sqlx::query("insert into public.tickets(uid,day,source,part,earned) values($1,2,'runner',$2,event_time()) on conflict do nothing")
                                .bind(uid).bind(pickup).execute(&mut *tx).await?;
                        }
                    }
                    points = Some(n);
                }
            }
            let mut r = round(&mut tx, id, uid).await?;
            r["points"] = json!(points);
            r
        }
        "profile" => {
            let nick = v["nick"].as_str().unwrap_or("").trim();
            if !(2..=24).contains(&nick.chars().count()) {
                return Err(bad("Никнейм – от 2 до 24 символов"));
            }
            sqlx::query("update users set nick=$2,location=$3,avatar=$4,sound=$5,motion=$6,haptic=$7,notify=$8 where id=$1").bind(uid).bind(nick).bind(v["location"].as_str().unwrap_or("").chars().take(40).collect::<String>()).bind(v["avatar"].as_str().unwrap_or("⚽").chars().take(4).collect::<String>()).bind(v["sound"]==true).bind(v["motion"]==true).bind(v["haptic"]==true).bind(v["notify"]==true).execute(&mut *tx).await?;
            if v["notify"] != true {
                sqlx::query("update jobs set status='cancelled' where uid=$1 and status='queued'")
                    .bind(uid)
                    .execute(&mut *tx)
                    .await?
            } else {
                Default::default()
            };
            json!(true)
        }
        "rating" => {
            let g = v["game"]
                .as_i64()
                .filter(|g| (1..=10).contains(g))
                .ok_or_else(|| bad("Выбери игру"))? as i32;
            sqlx::query_scalar(include_str!("rating.sql"))
                .bind(g)
                .bind(uid)
                .fetch_one(&mut *tx)
                .await?
        }
        _ => {
            if !a.admins.contains(&uid) {
                return Err(deny());
            }
            staff::act(&mut tx, op, uid, day, &v).await?
        }
    };
    tx.commit().await?;
    Ok(Json(out))
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    let name: String = sqlx::query_scalar("select current_database()")
        .fetch_one(&db)
        .await?;
    anyhow::ensure!(
        name.ends_with("_test") == cfg!(feature = "rehearsal"),
        "Rehearsal requires a database ending in _test"
    );
    sqlx::migrate!().run(&db).await?;
    let year: i32 = env::var("EVENT_YEAR").unwrap_or("2026".into()).parse()?;
    let seed = random();
    let y:i32=sqlx::query_scalar("insert into season(id,year,seed,commitment) values(true,$1,$2,$3) on conflict(id) do update set year=season.year returning year").bind(year).bind(&seed).bind(hash(&seed)).fetch_one(&db).await?;
    anyhow::ensure!(y == year, "Use a fresh database for the next event");
    let app = App {
        db,
        bot: env::var("BOT_TOKEN").unwrap_or_default(),
        admins: env::var("ADMIN_IDS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect(),
        limits: Default::default(),
    };
    anyhow::ensure!(
        cfg!(feature = "rehearsal") || !app.bot.is_empty(),
        "BOT_TOKEN required"
    );
    let a = app.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = staff::tick(&a).await {
                eprintln!("worker {e:?}")
            }
            tokio::time::sleep(Duration::from_secs(4)).await;
        }
    });
    let routes = Router::new()
        .route("/", get(assets::page))
        .fallback(get(assets::page))
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/public-proof", get(public_proof))
        .route("/api/login", post(login))
        .route("/api/{op}", post(rpc))
        .layer(DefaultBodyLimit::max(16384))
        .with_state(app);
    axum::serve(
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", env::var("PORT").unwrap_or_else(|_| "10000".into()))).await?,
        routes,
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod checks {
    use super::*;
    #[test]
    fn distribution() {
        let mut h = [0; 5];
        for n in 0..100 {
            h[(game::reward(n) - 1) as usize] += 1
        }
        assert_eq!(h, [40, 30, 20, 8, 2]);
        assert_eq!(20 * game::reward(99) + 10 * 5, 150)
    }
    #[test]
    fn auth() {
        let bot = "123:testing";
        let user = r#"{"id":42}"#;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let sign = |stamp: u64| {
            let key = Hmac::<Sha256>::new_from_slice(b"WebAppData")
                .unwrap()
                .chain_update(bot)
                .finalize()
                .into_bytes();
            let hash = hex::encode(
                Hmac::<Sha256>::new_from_slice(&key)
                    .unwrap()
                    .chain_update(format!("auth_date={stamp}\nuser={user}"))
                    .finalize()
                    .into_bytes(),
            );
            url::form_urlencoded::Serializer::new(String::new())
                .append_pair("auth_date", &stamp.to_string())
                .append_pair("user", user)
                .append_pair("hash", &hash)
                .finish()
        };
        assert_eq!(verified(&sign(now), bot).unwrap(), 42);
        assert!(verified(&sign(now), "wrong").is_err());
        assert!(verified(&sign(now - 602), bot).is_err());
        assert!(verified(&sign(now + 100), bot).is_err());
        assert!(verified(&(sign(now) + "&user=42"), bot).is_err())
    }
    #[test]
    fn ten_games() {
        for g in 1..=10 {
            for step in 0..4 {
                let mut c = game::scene(g, step);
                c["phase"] = json!(0);
                c["speed"] = json!(1.);
                let x = c["x"].as_f64().unwrap();
                let lane = c["lane"].as_i64().unwrap();
                let (a, ms) = match c["kind"].as_i64().unwrap() {
                    1 => (json!({"x":lane*33+15}), 1200),
                    2 => {
                        let k = [0.94, 1.45, 0.76][lane as usize];
                        (json!({"origin":true,"x":50.+(x-50.)/k,"y":80.-42./k}), 1200)
                    }
                    3..=5 => (json!({"choice":c["answer"]}), 800),
                    6 => (json!({"x":x}), 1100),
                    7 => {
                        let mut prev = [-1; 25];
                        prev[0] = 0;
                        let mut todo = vec![0];
                        while let Some(i) = todo.pop() {
                            for e in c["edges"].as_array().unwrap() {
                                let (a, b) = (e[0].as_i64().unwrap(), e[1].as_i64().unwrap());
                                let n = if i == a {
                                    b
                                } else if i == b {
                                    a
                                } else {
                                    continue;
                                };
                                if prev[n as usize] == -1 {
                                    prev[n as usize] = i;
                                    todo.push(n)
                                }
                            }
                        }
                        let mut p = vec![24];
                        while *p.last().unwrap() != 0 {
                            p.push(prev[*p.last().unwrap() as usize])
                        }
                        p.reverse();
                        (json!({"path":p}), 4000)
                    }
                    8 => (json!({"x":50.+34.*1f64.sin(),"y":32}), 1000),
                    9 => (json!({"x":x,"y":80}), 1540),
                    _ => unreachable!(),
                };
                assert!(game::score(&c, &a, ms, 440) > 0);
                assert_eq!(game::score(&c, &json!({}), ms, 0), 0);
                assert_eq!(game::score(&c, &a, 10, 440), 0)
            }
        }
    }
}
