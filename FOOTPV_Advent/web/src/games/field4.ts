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
  g.lineBetween(25, 45, 25, 320);
  ball(
    c.variant === 0
      ? Math.max(9, 125 - t * 82)
      : c.variant === 1
        ? Math.max(26, 125 - t * 82)
        : 150 + Math.sin(t * 2) * 82,
    168,
  );
}
