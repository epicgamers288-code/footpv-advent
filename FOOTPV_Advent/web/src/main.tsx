import { lazy, Suspense, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { api, login, tg, tone } from "./api";
import Profile, { Rules, file } from "./Profile";
import "./style.css";
const Match = lazy(() => import("./Match")),
  Admin = lazy(() => import("./Admin"));
const tabs = ["Календарь", "Колесо", "Рейтинг", "Билеты", "Профиль"],
  icons = [
    "M4 6h16v15H4z M4 10h16 M8 3v5 M16 3v5 M8 14h2 M14 14h2",
    "M12 3a9 9 0 1 0 .01 0 M12 3v9l8 4 M12 12l-7 5 M12 12l8-5",
    "M3 21h18 M5 18v-6h3v6z M11 18V5h3v13z M17 18V9h3v9z",
    "M3 6h18v4a2 2 0 0 0 0 4v4H3v-4a2 2 0 0 0 0-4z M15 8v2 M15 13v2",
    "M12 3a4 4 0 1 0 .01 0 M4 21v-2a8 6 0 0 1 16 0v2z",
  ],
  comments = [
    "Билет в копилке!",
    "Уже неплохо!",
    "Добротный результат.",
    "Вот это подача!",
    "Новогодняя пятёрочка!",
  ];
function Gift({ n, large = false }: { n: number; large?: boolean }) {
  return (
    <img
      className={"gift" + (large ? " large" : "")}
      alt=""
      src={"/gift-" + (n % 4) + ".webp"}
    />
  );
}
function Hero() {
  return (
    <div className="hero">
      <small>11 – 30 ДЕКАБРЯ　/　АДВЕНТ АКАДЕМИИ</small>
      <h1>
        ДО НОВОГО
        <br />
        <i>ГОЛА!</i>
      </h1>
      <Gift n={0} large />
      <p>20 ДНЕЙ. 10 ИГР. НОВОГОДНИЕ ПОДАРКИ.</p>
    </div>
  );
}
function App() {
  const [s, S] = useState<any>(),
    [p, P] = useState(0),
    [d, D] = useState(""),
    [msg, E] = useState(""),
    [busy, B] = useState(false),
    [test, T] = useState(1),
    [r, R] = useState<any>(),
    [angle, A] = useState(0),
    [spin, Z] = useState(false),
    [group, G] = useState(1),
    [rating, Y] = useState<any>(),
    [proof, F] = useState<any>();
  const refresh = async () => {
    const v = await api("state");
    S(v);
    return v;
  };
  async function run(fn: () => Promise<any>) {
    if (busy) return;
    B(true);
    E("");
    try {
      await fn();
    } catch (e) {
      E((e as Error).message);
    } finally {
      B(false);
    }
  }
  useEffect(() => {
    tg?.ready();
    tg?.expand();
    const t = new URLSearchParams(location.search).get("tab") || "";
    P(({ wheel: 1, rating: 2, tickets: 3 } as any)[t] || 0);
    if (t === "rules" || t === "games") D(t === "rules" ? t : "game");
    (async () => {
      if (sessionStorage.getItem("fp-session")) {
        try {
          await refresh();
          return;
        } catch {}
      }
      if (tg?.initData) {
        await login();
        await refresh();
      }
    })().catch((e) => E(e.message));
  }, []);
  useEffect(() => {
    if (!s || r) return;
    const id = setInterval(() => {
      if (!document.hidden) refresh().catch((e) => E(e.message));
    }, 15000);
    return () => clearInterval(id);
  }, [!!s, !!r]);
  useEffect(() => {
    if (p !== 2 || !s) return;
    let on = true;
    api("rating", { game: group })
      .then((v) => {
        if (on) Y(v);
      })
      .catch((e) => E(e.message));
    return () => {
      on = false;
    };
  }, [p, group, s?.now]);
  const won = s?.spins?.[s?.wheelDay];
  useEffect(() => {
    if (won && !spin)
      A((a) => Math.floor(a / 360) * 360 + 360 - (won - 0.5) * 72);
  }, [won, spin]);
  useEffect(() => {
    if (!spin) return;
    const t = setTimeout(() => Z(false), s.profile.motion ? 3300 : 0);
    return () => clearTimeout(t);
  }, [spin]);
  const head = (
    <header>
      <b>
        <img src="/white.png" /> FOOTPV
      </b>
      <span>ЗИМНИЙ СЕЗОН　{s?.year || 2026}</span>
    </header>
  );
  if (!s)
    return (
      <main>
        {head}
        <Hero />
        <section>
          <h2>Раздевалка</h2>
          <p>
            Открой календарь через бот академии. За ребёнка участвует родитель
            или законный представитель.
          </p>
          <a className="cta" href="https://t.me/FOOTPV_bot?start=advent">
            Открыть @FOOTPV_bot
          </a>
          {["localhost", "127.0.0.1"].includes(location.hostname) && (
            <fieldset>
              <legend>Локальная репетиция</legend>
              <label>
                Номер тестового игрока
                <input
                  type="number"
                  min="1"
                  max="99999"
                  value={test}
                  onChange={(e) => T(+e.target.value)}
                />
              </label>
              <button
                disabled={busy}
                onClick={() =>
                  run(async () => {
                    await login(test);
                    await refresh();
                  })
                }
              >
                Войти в тестовый сезон
              </button>
            </fieldset>
          )}
          <p role="alert">{msg}</p>
        </section>
      </main>
    );
  const game = s.game,
    t = s.tickets.length,
    at = s.attempts.filter((a: any) => a.game === game),
    active = at.find(
      (a: any) => !a.done && Date.parse(a.deadline) > Date.parse(s.now),
    ),
    gameTickets = s.tickets.filter((t: any) => t.source === "game").length;
  return (
    <main className={s.profile.motion ? "" : "still"}>
      {head}
      {r ? (
        <Suspense fallback={<p>Чистим поле от снега...</p>}>
          <Match
            initial={r}
            user={s.profile}
            exit={() => {
              R(null);
              void run(refresh);
            }}
          />
        </Suspense>
      ) : (
        <>
          <div className="identity">
            <b>
              {s.profile.avatar}　{s.profile.nick}
            </b>
            <span>▱　{t} бил.</span>
          </div>
          {p === 0 && (
            <>
              <Hero />
              <div className="line">
                <h2>Календарь</h2>
                <b>{s.opened.length} / 20</b>
              </div>
              <progress max={20} value={s.opened.length} />
              <p className="muted">
                {s.day < 1
                  ? "Старт 11 декабря в 09:00"
                  : s.day > 20
                    ? "Сезон завершён"
                    : "День " + s.day + " из 20"}{" "}
                · Минск
              </p>
              <div className="grid">
                {Array.from({ length: 20 }, (_, i) => {
                  const n = i + 1,
                    status = s.spins[n]
                      ? "Выполнена"
                      : s.opened.includes(n)
                        ? "Открыта"
                        : s.wheelDay === n
                          ? "Доступна"
                          : n < s.day
                            ? "Пропущена"
                            : "Закрыта";
                  return (
                    <button
                      className={"door " + (s.wheelDay === n ? "today" : "")}
                      key={n}
                      onClick={() =>
                        run(async () => {
                          if (s.wheelDay === n) {
                            await api("open", { day: n });
                            await refresh();
                            D("door");
                          } else D(status === "Закрыта" ? "locked" : "past");
                        })
                      }
                    >
                      <small>{10 + n} ДЕК</small>
                      <Gift n={i} />
                      <span>{status}</span>
                    </button>
                  );
                })}
              </div>
              {game && (
                <button className="cta matchlink" onClick={() => D("game")}>
                  {s.games[game - 1][0]}　↗
                </button>
              )}
              <button
                className="finalcard"
                onClick={() =>
                  run(async () => {
                    F(await (await fetch("/api/public-proof")).json());
                    D("proof");
                  })
                }
              >
                <small>31 ДЕКАБРЯ</small>
                <b>Большой розыгрыш　↗</b>
              </button>
            </>
          )}
          {p === 1 && (
            <section>
              <small>ЕЖЕДНЕВНО · 09:00–00:00</small>
              <h2>Колесо подарков</h2>
              <p>Одно вращение в день. Каждый сектор приносит билеты.</p>
              <div className="wheelwrap">
                <b className="pointer">▼</b>
                <div
                  className="wheel"
                  style={{
                    transform: "rotate(" + angle + "deg)",
                    transitionDuration:
                      spin && s.profile.motion ? "3.2s" : "0s",
                  }}
                >
                  {[1, 2, 3, 4, 5].map((n) => (
                    <b
                      key={n}
                      style={{
                        transform:
                          "rotate(" + (n - 0.5) * 72 + "deg) translateY(-90px)",
                      }}
                    >
                      {n}
                    </b>
                  ))}
                  <img src="/white.png" />
                </div>
              </div>
              {spin ? (
                <button onClick={() => Z(false)}>Пропустить анимацию</button>
              ) : won ? (
                <div className="reward">
                  +{won} ▱　{comments[won - 1]}
                </div>
              ) : (
                <button
                  className="cta"
                  disabled={busy || !s.wheelDay}
                  onClick={() =>
                    run(async () => {
                      const v = await api("spin", { day: s.wheelDay });
                      A(
                        (a) =>
                          Math.ceil(a / 360) * 360 +
                          1800 -
                          (v.reward - 0.5) * 72,
                      );
                      Z(true);
                      tone(true, s.profile.sound);
                      await refresh();
                    })
                  }
                >
                  {s.wheelDay ? "Крутить" : "Вращение закрыто"}
                </button>
              )}
              <button onClick={() => D("rules")}>Правила и вероятности</button>
            </section>
          )}
          {p === 2 && (
            <section>
              <small>ТУРНИРНАЯ ТАБЛИЦА</small>
              <h2>Рейтинг</h2>
              {s.games.length ? (
                <>
                  <select value={group} onChange={(e) => G(+e.target.value)}>
                    {s.games.map((g: any, i: number) => (
                      <option key={i} value={i + 1}>
                        {i + 1}. {g[0]}
                      </option>
                    ))}
                  </select>
                  <p>
                    {rating?.closed
                      ? "Итоговый зачёт"
                      : "Предварительный зачёт"}
                  </p>
                  {rating?.rows.map((x: any, i: number) => (
                    <div
                      className={"rank " + (x.place <= 5 ? "gold" : "")}
                      key={i}
                    >
                      <b>#{x.place}</b>
                      <span>
                        {x.avatar} {x.nick}
                        {x.mine ? " · Вы" : ""}
                      </span>
                      <b>{x.score}</b>
                      <small>{x.tickets} ▱</small>
                    </div>
                  ))}
                  {!rating?.rows.length && (
                    <p>Результаты появятся после завершения попыток.</p>
                  )}
                </>
              ) : (
                <p>Первая игра откроется 11 декабря.</p>
              )}
            </section>
          )}
          {p === 3 && (
            <section>
              <small>КОЛЛЕКЦИЯ УЧАСТНИКА</small>
              <h2>Мои билеты</h2>
              <div className="ticket">
                <small>FOOTPV　/　ADVENT</small>
                <b>{String(t).padStart(3, "0")}</b>Колесо {t - gameTickets}
                　·　Игры {gameTickets}
              </div>
              <h3>История начислений</h3>
              {s.tickets.map((t: any) => (
                <div className="rank" key={t.id}>
                  <b>№ {t.id}</b>
                  <span>
                    {t.source === "wheel"
                      ? "Колесо"
                      : "Игра · место " + t.place}
                  </span>
                  <small>{10 + t.day} дек.</small>
                </div>
              ))}
              {!t && <p>Здесь появятся номера твоих билетов.</p>}
            </section>
          )}
          {p === 4 && <Profile s={s} refresh={refresh} open={D} />}
          <nav>
            {tabs.map((n, i) => (
              <button
                key={n}
                className={p === i ? "sel" : ""}
                onClick={() => {
                  P(i);
                  E("");
                }}
              >
                <svg
                  aria-hidden="true"
                  width="22"
                  height="22"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d={icons[i]} />
                </svg>
                {n}
              </button>
            ))}
          </nav>
        </>
      )}
      {d && !r && (
        <div className="overlay">
          <article role="dialog" aria-modal="true">
            <button
              className="close"
              aria-label="Закрыть"
              onClick={() => D("")}
            >
              ×
            </button>
            {d === "admin" ? (
              <Suspense fallback="Загрузка">
                <Admin test={s.rehearsal} refresh={refresh} />
              </Suspense>
            ) : d === "rules" ? (
              <Rules />
            ) : d === "door" ? (
              <>
                <Gift n={s.day} />
                <h2>Дверца открыта!</h2>
                <button
                  className="cta"
                  onClick={() => {
                    P(1);
                    D("");
                  }}
                >
                  К колесу подарков
                </button>
                <button onClick={() => D("game")}>На игровое поле</button>
              </>
            ) : d === "game" ? (
              <>
                <h2>{s.games[game - 1]?.[0] || "Игра закрыта"}</h2>
                <p>{s.games[game - 1]?.[1]}</p>
                <p>
                  Использовано {at.length} из 3 попыток за два дня. Длительность
                  – {s.games[game - 1]?.[2] || 0} с.
                </p>
                <button
                  className="cta"
                  disabled={busy || !game || (!active && at.length >= 3)}
                  onClick={() =>
                    run(async () => {
                      const v = await api("start", { game });
                      v.name = s.games[game - 1][0];
                      v.previousBest = Math.max(
                        0,
                        ...at
                          .filter((a: any) => a.done && !a.flagged)
                          .map((a: any) => a.score),
                      );
                      R(v);
                      D("");
                    })
                  }
                >
                  {active ? "Продолжить попытку" : "Начать зачётную попытку"}
                </button>
              </>
            ) : d === "proof" ? (
              <>
                <h2>Большой розыгрыш</h2>
                <p>31 декабря · один подарок на участника</p>
                {proof?.prizes?.map((x: string, i: number) => (
                  <p key={i}>
                    {i + 1}. {x}
                  </p>
                ))}
                {proof?.winners?.map((w: any) => (
                  <div className="reward" key={w.ticket}>
                    № {w.ticket} · {w.prize}
                  </div>
                ))}
                <small>Обязательство SHA-256</small>
                <code>{proof?.commitment}</code>
                {proof?.digest && (
                  <>
                    <small>Хеш списка билетов</small>
                    <code>{proof.digest}</code>
                    <button
                      onClick={() =>
                        file(
                          new Blob([JSON.stringify(proof, null, 2)], {
                            type: "application/json",
                          }),
                          "draw.json",
                        )
                      }
                    >
                      Скачать протокол
                    </button>
                  </>
                )}
              </>
            ) : (
              <>
                <h2>{d === "locked" ? "Пока секрет" : "Дверца в архиве"}</h2>
                <p>
                  {d === "locked"
                    ? "Рановато пришёл. Даже тренера ещё нет."
                    : "Билеты сохранены в коллекции."}
                </p>
              </>
            )}
            {msg && <p role="alert">{msg}</p>}
          </article>
        </div>
      )}
      {msg && !d && (
        <button className="error" onClick={() => E("")}>
          {msg} ×
        </button>
      )}
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<App />);
