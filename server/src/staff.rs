use super::*;
async fn queue(tx: &mut Tx<'_>, key: &str, body: &str) -> R<()> {
    sqlx::query("insert into jobs(uid,kind,body) select id,$1,$2 from users where notify on conflict do nothing").bind(key).bind(body).execute(&mut **tx).await?;
    Ok(())
}
pub async fn tick(a: &App) -> R<()> {
    let mut tx = a.db.begin().await?;
    sqlx::query("select pg_advisory_xact_lock(817402)")
        .execute(&mut *tx)
        .await?;
    sqlx::query("select settle(event_time())")
        .execute(&mut *tx)
        .await?;
    let day: Option<i32> = sqlx::query_scalar("select wheel_day(event_time())")
        .fetch_one(&mut *tx)
        .await?;
    if let Some(d) = day {
        queue(
            &mut tx,
            &format!("day-{d}"),
            if d % 2 == 1 {
                "Открыты новая игра, дверца и колесо. Бутсы на месте? Погнали!"
            } else {
                "Дверца и колесо уже открыты. Вчерашняя игра продолжается!"
            },
        )
        .await?
    }
    sqlx::query("delete from sessions where expires<now()")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    if !cfg!(feature = "rehearsal") {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .unwrap();
        let jobs=sqlx::query("update jobs j set status='sending' from (select uid,kind from jobs join users on id=uid where notify and status='queued' order by kind limit 8 for update of jobs skip locked) b where j.uid=b.uid and j.kind=b.kind returning j.uid,j.kind,j.body").fetch_all(&a.db).await?;
        for j in jobs {
            let uid: i64 = j.get("uid");
            let reply = client
                .post(format!("https://api.telegram.org/bot{}/sendMessage", a.bot))
                .json(&json!({"chat_id":uid,"text":j.get::<String,_>("body")}))
                .send()
                .await;
            let status = match reply {
                Ok(r) if r.status().is_success() => "sent",
                Ok(r) if r.status().as_u16() == 429 => "queued",
                _ => "uncertain",
            };
            sqlx::query("update jobs set status=$3 where uid=$1 and kind=$2")
                .bind(uid)
                .bind(j.get::<String, _>("kind"))
                .bind(status)
                .execute(&a.db)
                .await?;
        }
    }
    Ok(())
}
pub async fn act(tx: &mut Tx<'_>, op: &str, uid: i64, day: i32, v: &V) -> R<V> {
    let result=match op{
 "admin"=>return Ok(sqlx::query_scalar(include_str!("admin.sql")).fetch_one(&mut **tx).await?),
 "clock" if cfg!(feature="rehearsal")=>{let t=v["utc"].as_str().ok_or_else(||bad("Дата обязательна"))?;sqlx::query("update season set shift=$1::text::timestamptz-clock_timestamp()").bind(t).execute(&mut **tx).await?;json!(true)},
 "people"=>return Ok(sqlx::query_scalar("select coalesce(jsonb_agg(to_jsonb(s)),'[]') from(select nick,location,alias,(select count(*) from tickets where uid=u.id and valid) tickets from users u where nick ilike '%'||$1||'%' order by nick,id limit 100)s").bind(v["search"].as_str().unwrap_or("")).fetch_one(&mut **tx).await?),
 "history"=>return Ok(sqlx::query_scalar("select jsonb_build_object('spins',(select coalesce(jsonb_agg(to_jsonb(s)-'uid' order by day),'[]') from spins s where uid=u.id),'attempts',(select coalesce(jsonb_agg(jsonb_build_object('id',id,'game',game,'score',score,'flagged',flagged)),'[]') from attempts where uid=u.id),'tickets',(select coalesce(jsonb_agg(id order by id),'[]') from tickets where uid=u.id)) from users u where alias::text=$1").bind(v["alias"].as_str().unwrap_or("")).fetch_one(&mut **tx).await?),
 "prize"=>{if day>0{return Err(bad("Призы фиксируются до начала сезона"))}let title=v["title"].as_str().filter(|s|!s.trim().is_empty()&&s.len()<180).ok_or_else(||bad("Название приза"))?;sqlx::query("insert into prizes(title,request) values($1,$2::text::uuid) on conflict(request) do nothing").bind(title).bind(v["request"].as_str().unwrap_or("")).execute(&mut **tx).await?;json!(true)},
 "review"=>{if v["reason"].as_str().is_none_or(|s|s.trim().len()<6){return Err(bad("Укажите основание решения"))}let row=sqlx::query("update attempts set flagged=$2 where id::text=$1 and deadline<=event_time() and event_time()<closes(game) returning id").bind(v["id"].as_str().unwrap_or("")).bind(v["flagged"]==true).fetch_optional(&mut **tx).await?;if row.is_none(){return Err(bad("Проверка этой попытки закрыта"))}json!(true)},
 "broadcast"=>{let text=v["message"].as_str().filter(|s|!s.trim().is_empty()&&s.len()<3000).ok_or_else(||bad("Введите текст сообщения"))?;queue(tx,&format!("message-{}",hash(text)),text).await?;json!(true)},
 "freeze"|"draw"=>{if day!=21{return Err(bad("Розыгрыш проходит 31 декабря"))}let proof:V=sqlx::query_scalar("select data from proof").fetch_one(&mut **tx).await?;if !proof["winners"].is_null(){return Ok(proof)}
 if proof["prizes"].as_array().unwrap().is_empty(){return Err(bad("Призы ещё не заданы"))}
 if op=="freeze"{if !proof["snapshot"].is_null(){return Ok(proof)}sqlx::query("select settle(event_time())").execute(&mut **tx).await?;
 let pool:V=sqlx::query_scalar("select coalesce(jsonb_agg(jsonb_build_array(t.id,u.alias::text) order by t.id),'[]') from tickets t join users u on u.id=t.uid where valid").fetch_one(&mut **tx).await?;if pool.as_array().unwrap().is_empty(){return Err(bad("Нет действительных билетов"))}let digest=hash(&pool.to_string());sqlx::query("update season set snapshot=$1,digest=$2 where snapshot is null").bind(pool).bind(digest).execute(&mut **tx).await?;}else{
 let mut pool=proof["snapshot"].as_array().cloned().ok_or_else(||bad("Сначала зафиксируйте список билетов"))?;let seed:String=sqlx::query_scalar("select seed from season").fetch_one(&mut **tx).await?;let digest=proof["digest"].as_str().unwrap();let mut counter=0u64;let mut winners=vec![];for prize in proof["prizes"].as_array().unwrap(){if pool.is_empty(){break}let n=pool.len()as u64;let bound=u64::MAX-u64::MAX%n;let index=loop{let h=Sha256::digest(format!("{seed}:{digest}:{counter}"));counter+=1;let k=u64::from_be_bytes(h[..8].try_into().unwrap());if k<bound{break(k%n)as usize}};let w=pool[index].clone();winners.push(json!({"prize":prize,"ticket":w[0],"owner":w[1]}));pool.retain(|t|t[1]!=w[1]);}
 sqlx::query("update season set winners=$1 where winners is null").bind(json!(winners)).execute(&mut **tx).await?;queue(tx,"final","Опубликованы итоги большого розыгрыша FOOTPV!").await?}
 sqlx::query_scalar("select data from proof").fetch_one(&mut **tx).await?},
 _=>return Err(bad("Неизвестная операция"))};
    sqlx::query("insert into audit(actor,action,detail) values($1,$2,$3)")
        .bind(uid)
        .bind(op)
        .bind(v)
        .execute(&mut **tx)
        .await?;
    Ok(result)
}
