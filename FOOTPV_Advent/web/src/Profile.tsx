import { useState } from "react";
import { api, tg } from "./api";
export function file(data: Blob, name: string) {
  const url = URL.createObjectURL(data),
    a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
async function card(n: number, doors: number) {
  const c = document.createElement("canvas");
  c.width = 1080;
  c.height = 1350;
  const x = c.getContext("2d")!;
  const g = x.createLinearGradient(0, 0, 1000, 1350);
  g.addColorStop(0, "#164659");
  g.addColorStop(1, "#08172b");
  x.fillStyle = g;
  x.fillRect(0, 0, 1080, 1350);
  const logo = new Image();
  logo.src = "/white.png";
  await logo.decode();
  x.drawImage(logo, 58, 72, 105, 105);
  x.fillStyle = "white";
  x.font = "bold 52px Arial";
  x.fillText("FOOTPV", 175, 147);
  x.font = "italic bold 116px Arial";
  x.fillText("ДО НОВОГО", 73, 480);
  x.fillStyle = "#8ce9d7";
  x.fillText("ГОЛА!", 73, 600);
  x.fillStyle = "#ffcf80";
  x.font = "bold 175px Arial";
  x.fillText(String(n), 73, 945);
  x.font = "46px Arial";
  x.fillText("билетов в коллекции", 73, 1025);
  x.fillText("Открыто дверец  " + doors + " / 20", 73, 1180);
  c.toBlob((b) => {
    if (b) file(b, "FOOTPV-result.png");
  });
}
export default function Profile({
  s,
  refresh,
  open,
}: {
  s: any;
  refresh: () => Promise<any>;
  open: (d: string) => void;
}) {
  const [p, S] = useState(s.profile),
    [msg, M] = useState("");
  const played = s.attempts.filter(
      (a: any) => a.done && a.step > 0 && !a.flagged,
    ),
    done = new Set(played.map((a: any) => a.game)),
    t = s.tickets.length;
  const badges: [string, boolean][] = [
    ["Первый билет", t > 0],
    ["Первая открытая дверца", s.opened.length > 0],
    ["Первый топ-5", s.tickets.some((t: any) => t.place && t.place <= 5)],
    ["Бутсы на месте", done.has(3)],
    ["Стена", done.has(6)],
    ["Девяточка", done.has(8)],
    ["Футбольный интеллект", done.has(4) && done.has(5)],
    ["10 пройденных игр", done.size === 10],
    ["20 открытых дверец", s.opened.length === 20],
    ...[25, 50, 100].map(
      (n) => ["Коллекция из " + n + " билетов", t >= n] as [string, boolean],
    ),
    ["Максимальная коллекция", t === 150],
  ];
  return (
    <section>
      <small>ЛИЧНЫЙ КАБИНЕТ</small>
      <h2>Профиль</h2>
      <form
        onSubmit={async (e) => {
          e.preventDefault();
          try {
            if (
              p.notify &&
              !s.profile.notify &&
              tg?.initData &&
              tg.requestWriteAccess
            ) {
              const ok = await new Promise<boolean>((res) =>
                tg.requestWriteAccess(res),
              );
              if (!ok) throw Error("Разрешение на уведомления не получено");
            }
            await api("profile", p);
            await refresh();
            M("Профиль сохранён");
          } catch (e) {
            M((e as Error).message);
          }
        }}
      >
        {[
          ["nick", "Игровой никнейм"],
          ["location", "Локация FOOTPV"],
        ].map(([k, l]) => (
          <label key={k}>
            {l}
            <input
              value={p[k]}
              maxLength={k === "nick" ? 24 : 40}
              onChange={(e) => S({ ...p, [k]: e.target.value })}
            />
          </label>
        ))}
        <label>
          Аватар
          <select
            value={p.avatar}
            onChange={(e) => S({ ...p, avatar: e.target.value })}
          >
            {["⚽", "🧤", "🦊", "🐻", "🥅"].map((a) => (
              <option key={a}>{a}</option>
            ))}
          </select>
        </label>
        {[
          ["notify", "Уведомления в Telegram"],
          ["sound", "Звуки"],
          ["motion", "Анимации"],
          ["haptic", "Вибрация"],
        ].map(([k, l]) => (
          <label className="toggle" key={k}>
            {l}
            <input
              type="checkbox"
              checked={p[k]}
              onChange={(e) => S({ ...p, [k]: e.target.checked })}
            />
          </label>
        ))}
        <button className="cta">Сохранить</button>
      </form>
      <p role="status">{msg}</p>
      <h3>Мои результаты</h3>
      <p>
        {done.size} из 10 игр · {s.opened.length} из 20 дверец
      </p>
      {s.games.map((g: any, i: number) => {
        const a = played.filter((v: any) => v.game === i + 1);
        return a.length ? (
          <div key={i} className="line">
            {g[0]}
            <b>{Math.max(...a.map((v: any) => v.score))}</b>
          </div>
        ) : null;
      })}
      <h3>Достижения</h3>
      <div className="badges">
        {badges.map(([n, ok]) => (
          <span key={n} className={ok ? "earned" : ""}>
            {ok ? "★" : "☆"} {n}
          </span>
        ))}
      </div>
      <button
        onClick={() => card(t, s.opened.length).catch((e) => M(e.message))}
      >
        Скачать карточку результата
      </button>
      <button onClick={() => open("rules")}>Правила</button>
      {s.admin && (
        <button onClick={() => open("admin")}>Панель организатора</button>
      )}
    </section>
  );
}
export function Rules() {
  return (
    <>
      <h2>Правила участия</h2>
      <p>
        11–30 декабря по минскому времени. Участие бесплатное. За ребёнка
        участвует родитель или законный представитель через свой аккаунт.
      </p>
      <h3>Колесо</h3>
      <p>
        Одно вращение с 09:00 до полуночи. Вероятности для 1, 2, 3, 4 и 5
        билетов – 40%, 30%, 20%, 8% и 2%. Каждый сектор приносит награду.
      </p>
      <h3>Игры и рейтинг</h3>
      <p>
        Игры открываются в 09:00 11, 13, 15, 17, 19, 21, 23, 25, 27 и 29
        декабря. На каждую игру – три попытки до конца следующего дня. Сервер
        учитывает лучшую завершённую попытку с игровым действием. Сначала
        сравниваются очки, затем точные действия. При полном равенстве место
        общее.
      </p>
      <p>
        Места 1–5, 6–10, 11–20, 21–30 и остальные получают 5, 4, 3, 2 и 1 билет
        соответственно.
      </p>
      <h3>Большой розыгрыш</h3>
      <p>
        31 декабря все действительные билеты участвуют на равных. После выигрыша
        остальные билеты владельца выбывают. Билеты бесплатные, непередаваемые,
        без денежного эквивалента. Максимум за сезон – 150.
      </p>
      <h3>Данные участника</h3>
      <p>
        Храним идентификатор Telegram, игровой профиль, действия и билеты. В
        рейтинге видны игровой никнейм и аватар. Уведомления включаются в
        профиле. Вопросы и обращения по данным принимает академия.
      </p>
      <a href="https://footpv.by/">footpv.by</a> ·{" "}
      <a href="https://www.instagram.com/footpv/">Instagram</a> ·{" "}
      <a href="https://t.me/footpv_by">Telegram</a> ·{" "}
      <a href="https://vk.com/footpvby">ВКонтакте</a>
    </>
  );
}
