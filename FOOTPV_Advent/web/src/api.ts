export const tg = (window as any).Telegram?.WebApp;
let token = sessionStorage.getItem("fp-session") || "";
export async function api(op: string, data: object = {}) {
  const r = await fetch("/api/" + op, {
    method: "POST",
    headers: {
      Authorization: "Bearer " + token,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  });
  let v;
  try {
    v = await r.json();
  } catch {
    throw Error("Мяч улетел за пределы поля. Восстанавливаем связь.");
  }
  if (!r.ok) throw Error(v.error || "Ошибка запроса");
  return v;
}
export async function login(testUser?: number) {
  token = (
    await api(
      "login",
      testUser ? { testUser } : { initData: tg?.initData || "" },
    )
  ).token;
  sessionStorage.setItem("fp-session", token);
}
let ctx: AudioContext | undefined;
export function tone(win: boolean, on: boolean) {
  if (!on) return;
  try {
    const a = (ctx ??= new AudioContext());
    void a.resume();
    const o = a.createOscillator(),
      g = a.createGain();
    o.frequency.value = win ? 660 : 250;
    g.gain.setValueAtTime(0.04, a.currentTime);
    g.gain.exponentialRampToValueAtTime(0.001, a.currentTime + 0.14);
    o.connect(g);
    g.connect(a.destination);
    o.start();
    o.stop(a.currentTime + 0.15);
  } catch {}
}
