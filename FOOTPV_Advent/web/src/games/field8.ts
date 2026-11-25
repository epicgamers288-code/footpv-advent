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
  g.lineStyle(3, 0xffffff);
  g.strokeRect(20, 60, 260, 114);
  const x = (50 + 34 * Math.sin(t + phase)) * 3;
  g.fillStyle(0xf3a463);
  g.fillCircle(x, 115, 18);
  g.lineStyle(3, 0xffdf9b);
  g.strokeCircle(x, 115, 9);
  ball(150, 290);
}
