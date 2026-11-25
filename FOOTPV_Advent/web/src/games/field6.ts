import type { Frame } from "./frame";
export default function draw({
  g,
  c,
  t,
  phase,
  path,
  ball,
  player,
  glide,
}: Frame) {
  g.lineStyle(4, 0xffffff);
  g.strokeRect(15, 253, 270, 72);
  for (let i = 0; i < 9; i++) {
    g.lineStyle(1, 0xffffff, 0.4);
    g.lineBetween(16 + i * 33, 253, 16 + i * 33, 325);
  }
  ball(150 + (c.x * 3 - 150) * Math.min(1, t), 36 + Math.min(1, t / 1.7) * 245);
}
