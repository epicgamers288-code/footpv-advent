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
  g.fillStyle(0x0b2034);
  g.fillRoundedRect(45, 122, 210, 120, 20);
  g.lineStyle(6, 0x61cbcf);
  g.strokeRoundedRect(94, 98, 111, 56, 22);
  g.lineBetween(57, 150, 241, 150);
  ball(150, 195);
}
