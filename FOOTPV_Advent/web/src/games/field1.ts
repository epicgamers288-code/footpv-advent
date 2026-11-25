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
  const y = 40 + Math.min(t, 2.65) * 92;
  for (let i = 0; i < 3; i++)
    if (i !== c.lane) {
      g.fillStyle(0xff9452);
      g.fillTriangle(
        38 + i * 100,
        y + 12,
        62 + i * 100,
        y + 12,
        50 + i * 100,
        y - 13,
      );
      g.fillStyle(0xffe3c8);
      g.fillRect(45 + i * 100, y - 1, 10, 3);
    }
  g.lineStyle(3, 0x8ae9d6);
  g.lineBetween(10, 255, 290, 255);
  ball(150, 285);
}
