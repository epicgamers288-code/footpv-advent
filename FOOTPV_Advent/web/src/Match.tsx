import { useEffect, useRef, useState } from "react";
import Phaser from "phaser";
import type { Frame } from "./games/frame";
import { api, tg, tone } from "./api";
const modules = import.meta.glob<{ default: (f: Frame) => void }>(
  "./games/field*.ts",
);
export default function Match({
  initial,
  user,
  exit,
}: {
  initial: any;
  user: any;
  exit: () => void;
}) {
  const host = useRef<HTMLDivElement>(null),
    round = useRef(initial),
    clock = useRef(performance.now()),
    busy = useRef(false),
    path = useRef([0]),
    live = useRef(true);
  const [r, S] = useState(initial),
    [msg, M] = useState(""),
    [left, L] = useState(0);
  const elapsed = () => performance.now() - clock.current;
  async function send(input: object, finish = false) {
    if (busy.current || round.current.done) return;
    busy.current = true;
    try {
      const prev = round.current;
      const n = await api(finish ? "finish" : "action", {
        id: prev.id,
        step: prev.step,
        ...input,
      });
      if (!live.current) return;
      round.current = n;
      clock.current = performance.now();
      S(n);
      path.current = [0];
      if (n.points != null) {
        M(
          n.points
            ? "+" +
                n.points +
                (n.combo > 1 ? " · Серия ×" + n.combo : " · Точно!")
            : "Следующий эпизод",
        );
        tone(n.points > 0, user.sound);
        if (user.haptic && n.points > 0)
          tg?.HapticFeedback?.impactOccurred("light");
      }
    } catch (e) {
      if (live.current) M((e as Error).message);
    } finally {
      busy.current = false;
    }
  }
  useEffect(() => {
    live.current = true;
    const render = new Map<number, (f: Frame) => void>(),
      loading = new Set<number>();
    let origin = false,
      down = 0,
      shot: { x: number; y: number; start: number } | null = null;
    let charge: Promise<any> | null = null;
    const timer = setInterval(() => {
      const v = round.current;
      if (v.done) return;
      const ms = Date.parse(v.deadline) - Date.parse(v.now) - elapsed();
      L(Math.max(0, Math.ceil(ms / 1000)));
      if (ms <= 0) void send({}, true);
    }, 220);
    class Pitch extends Phaser.Scene {
      pen!: Phaser.GameObjects.Graphics;
      create() {
        this.pen = this.add.graphics();
        this.input.on("pointerdown", (p: Phaser.Input.Pointer) => {
          const v = round.current;
          down = performance.now();
          origin = Math.hypot(p.x - 150, p.y - 288) < 40;
          if (v.scene.kind === 8 && !busy.current)
            charge = api("charge", { id: v.id, step: v.step }).catch(
              () => null,
            );
        });
        this.input.on("pointerup", async (p: Phaser.Input.Pointer) => {
          const v = round.current,
            c = v.scene;
          if (busy.current || v.done || c.options || shot) return;
          const x = p.x / 3,
            y = p.y / 3.6;
          if (c.kind === 2) {
            if (!origin) return;
            const k = [0.94, 1.45, 0.76][c.lane];
            shot = {
              x: 150 + (p.x - 150) * k,
              y: 288 + (p.y - 288) * k,
              start: performance.now(),
            };
            this.time.delayedCall(550, () => {
              shot = null;
              void send({ x, y, origin: true });
            });
            return;
          }
          if (c.kind === 7) {
            const cell = Math.floor(p.x / 60) + 5 * Math.floor((p.y - 30) / 60),
              prev = path.current.at(-1)!;
            if (
              c.edges.some(
                ([a, b]: number[]) =>
                  (a === prev && b === cell) || (b === prev && a === cell),
              )
            ) {
              path.current.push(cell);
              if (cell === 24) void send({ path: path.current });
            }
            return;
          }
          if (c.kind === 8 && charge) {
            await charge;
            charge = null;
          }
          void send({ x, y });
        });
      }
      update() {
        const v = round.current,
          c = v.scene,
          g = this.pen;
        g.clear();
        g.fillStyle(0x123e4c);
        g.fillRect(0, 0, 300, 360);
        for (let i = 0; i < 6; i += 2) {
          g.fillStyle(0x194b55);
          g.fillRect(0, i * 60, 300, 60);
        }
        g.lineStyle(1, 0x95d3d1, 0.6);
        g.strokeRect(10, 10, 280, 340);
        g.strokeCircle(150, 180, 43);
        g.lineBetween(10, 180, 290, 180);
        g.strokeRect(72, 10, 156, 45);
        const ball = (x: number, y: number) => {
          g.fillStyle(0x071324, 0.5);
          g.fillEllipse(x + 2, y + 8, 20, 8);
          g.fillStyle(0xffffff);
          g.fillCircle(x, y, 9);
          g.fillStyle(0x193957);
          g.fillCircle(x + 1, y - 1, 3.4);
        };
        const player = (x: number, y: number, col: number) => {
          g.fillStyle(col);
          g.fillCircle(x, y, 11);
          g.lineStyle(2, 0xffffff);
          g.strokeCircle(x, y, 11);
        };
        const f = render.get(c.kind);
        if (f)
          f({
            g,
            c,
            t:
              (elapsed() + Date.parse(v.now) - Date.parse(v.issued)) *
              0.001 *
              c.speed,
            phase: c.phase / 10,
            path: path.current,
            ball,
            player,
            glide: shot
              ? {
                  ...shot,
                  f:
                    1 -
                    Math.pow(
                      1 - Math.min(1, (performance.now() - shot.start) / 550),
                      2,
                    ),
                }
              : null,
          });
        else if (!loading.has(c.kind)) {
          loading.add(c.kind);
          modules["./games/field" + c.kind + ".ts"]()
            .then((m) => render.set(c.kind, m.default))
            .catch(() => loading.delete(c.kind));
        }
        if (c.kind === 8 && this.input.activePointer.isDown) {
          const ms = performance.now() - down;
          g.fillStyle(ms >= 180 && ms <= 900 ? 0x76efd2 : 0xe5996f);
          g.fillRect(20, 338, Math.min(260, ms / 4), 7);
        }
        if (c.kind === 2 && origin && this.input.activePointer.isDown) {
          g.lineStyle(2, 0xffdc8a);
          g.lineBetween(
            150,
            288,
            this.input.activePointer.x,
            this.input.activePointer.y,
          );
        }
      }
    }
    const instance = new Phaser.Game({
      type: Phaser.CANVAS,
      width: 300,
      height: 360,
      parent: host.current!,
      audio: { noAudio: true },
      scene: Pitch,
      fps: {
        target:
          navigator.hardwareConcurrency > 0 &&
          navigator.hardwareConcurrency <= 2
            ? 40
            : 60,
      },
    });
    return () => {
      live.current = false;
      clearInterval(timer);
      instance.destroy(true);
    };
  }, []);
  return (
    <section>
      <small>ИГРОВАЯ АРЕНА</small>
      <h2>{r.done ? "Финальный свисток!" : initial.name}</h2>
      <div className="line">
        <b>{r.score} очков</b>
        <b>{left} с</b>
      </div>
      {r.done && r.score > initial.previousBest && (
        <p className="reward">Личный рекорд! Скауты уже выезжают.</p>
      )}
      <div className="pitch" ref={host} hidden={r.done} />
      {!r.done && (
        <>
          <p>
            {r.scene.q || r.scene.tip}
            {r.scene.kind === 2
              ? " Покрытие – " + ["снег", "лёд", "лужа"][r.scene.lane] + "."
              : ""}
          </p>
          {r.scene.kind === 5 && (
            <small>
              С трибуны слышно «{["Бей!", "Сам!", "Пасуй!"][r.scene.lane]}»
            </small>
          )}
          <div className="choices">
            {r.scene.options?.map((o: string, i: number) => (
              <button key={o} onClick={() => send({ choice: i })}>
                {o}
              </button>
            ))}
          </div>
        </>
      )}
      <p aria-live="polite">{msg}</p>
      <button onClick={exit}>{r.done ? "В календарь" : "Свернуть игру"}</button>
    </section>
  );
}
