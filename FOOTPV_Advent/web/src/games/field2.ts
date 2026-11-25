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
  const color = [0xbcd9de, 0x78cbee, 0x3e729e][c.lane];
  g.fillStyle(color, 0.7);
  g.fillEllipse(148, 180, 264, 207);
  g.lineStyle(4, 0xffcf6b);
  g.strokeCircle(c.x * 3, 137, 23);
  ball(
    glide ? 150 + (glide.x - 150) * glide.f : 150,
    glide ? 288 + (glide.y - 288) * glide.f : 288,
  );
}
