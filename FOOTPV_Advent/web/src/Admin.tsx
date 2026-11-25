import { useEffect, useState } from "react";
import { api } from "./api";
export default function Admin({
  test,
  refresh,
}: {
  test: boolean;
  refresh: () => Promise<any>;
}) {
  const [a, S] = useState<any>(),
    [err, E] = useState(""),
    [busy, B] = useState(false),
    [date, D] = useState("2026-12-11T09:00"),
    [title, T] = useState(""),
    [message, M] = useState(""),
    [search, Q] = useState(""),
    [users, U] = useState<any[]>([]),
    [history, H] = useState<any>(),
    [reason, J] = useState("");
  useEffect(() => {
    api("admin")
      .then(S)
      .catch((e) => E(e.message));
  }, []);
  async function act(op: string, v: object) {
    if (busy) return;
    B(true);
    try {
      await api(op, v);
      S(await api("admin"));
      await refresh();
      E("Сохранено");
    } catch (e) {
      E((e as Error).message);
    } finally {
      B(false);
    }
  }
  return (
    <>
      <h2>Организатор</h2>
      <p role="status">{err}</p>
      {a && (
        <>
          <p>Участников {a.users}</p>
          {test && (
            <fieldset>
              <legend>Репетиция · время Минска</legend>
              <input
                type="datetime-local"
                value={date}
                onChange={(e) => D(e.target.value)}
              />
              <button
                onClick={() =>
                  act("clock", {
                    utc: new Date(date + ":00+03:00").toISOString(),
                  })
                }
              >
                Перевести часы
              </button>
            </fieldset>
          )}
          <details>
            <summary>Участники и их билеты</summary>
            <input
              value={search}
              placeholder="Никнейм"
              onChange={(e) => Q(e.target.value)}
            />
            <button
              onClick={() =>
                api("people", { search })
                  .then(U)
                  .catch((e) => E(e.message))
              }
            >
              Найти
            </button>
            {users.map((u) => (
              <button
                key={u.alias}
                onClick={() =>
                  api("history", { alias: u.alias })
                    .then(H)
                    .catch((e) => E(e.message))
                }
              >
                {u.nick} · {u.tickets} бил.
              </button>
            ))}
            {history && (
              <>
                <p>Билеты {history.tickets.join(", ")}</p>
                {history.spins.map((s: any) => (
                  <div key={s.day}>
                    День {s.day} · колесо +{s.reward}
                  </div>
                ))}
                {history.attempts.map((r: any) => (
                  <div key={r.id}>
                    Игра {r.game} · {r.score} очков
                  </div>
                ))}
              </>
            )}
          </details>
          <h3>Активность по дням</h3>
          {a.days.map(([d, n, w, t]: number[]) => (
            <div className="chart" key={d}>
              <span>{d + 10} дек.</span>
              <meter min={0} max={Math.max(1, a.users)} value={n} />
              <span>
                {w} вращ. · {t} бил.
              </span>
            </div>
          ))}
          {[
            ["games", "Попытки по играм"],
            ["locations", "Участники по локациям"],
            ["wheel", "Награды колеса"],
            ["jobs", "Уведомления"],
          ].map(([k, l]) => (
            <details key={k}>
              <summary>{l}</summary>
              {Object.entries(a[k]).map(([n, v]) => (
                <div className="chart" key={n}>
                  {n || "Не указана"}
                  <meter
                    min={0}
                    max={Math.max(1, ...Object.values(a[k]).map(Number))}
                    value={Number(v)}
                  />
                  {String(v)}
                </div>
              ))}
            </details>
          ))}
          <h3>Призы по порядку</h3>
          <ol>
            {a.prizes.map((v: string, i: number) => (
              <li key={i}>{v}</li>
            ))}
          </ol>
          <input
            value={title}
            placeholder="Название приза"
            onChange={(e) => T(e.target.value)}
          />
          <button
            disabled={busy}
            onClick={() =>
              act("prize", { title, request: crypto.randomUUID() })
            }
          >
            Добавить приз
          </button>
          <button onClick={() => act("freeze", {})}>
            Зафиксировать билеты
          </button>
          <button
            onClick={() => {
              if (confirm("Провести единственный розыгрыш сезона?"))
                act("draw", {});
            }}
          >
            Провести розыгрыш
          </button>
          <h3>Проверка результатов</h3>
          <input
            value={reason}
            placeholder="Основание решения"
            onChange={(e) => J(e.target.value)}
          />
          {a.review.map((r: any) => (
            <button
              key={r.id}
              onClick={() =>
                act("review", { id: r.id, flagged: !r.flagged, reason })
              }
            >
              {r.nick} · {r.score} · {r.flagged ? "Восстановить" : "Исключить"}
            </button>
          ))}
          <h3>Рассылка подписчикам</h3>
          <input value={message} onChange={(e) => M(e.target.value)} />
          <button
            onClick={() => {
              if (confirm("Добавить сообщение в очередь?"))
                act("broadcast", { message });
            }}
          >
            Добавить в очередь
          </button>
          <details>
            <summary>Журнал действий</summary>
            {a.audit.map((r: any, i: number) => (
              <p key={i}>
                {r.created} · {r.action}
              </p>
            ))}
          </details>
        </>
      )}
    </>
  );
}
