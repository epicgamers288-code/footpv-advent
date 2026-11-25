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
  g.fillStyle(0xffcf6b, 0.22);
  g.fillRect(10, 187, 280, 114);
  const f = (t * 0.65 + phase) % 1;
  ball(c.x * 3, (80 - 240 * f * (1 - f)) * 3.6);
}
