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
  player(144, 201, 0x59dbce);
  if (c.variant === 1) {
    player(136, 145, 0xeb8f80);
    player(163, 125, 0xeb8f80);
    player(54, 87, 0x59dbce);
  }
  if (c.variant === 0) player(129, 247, 0xeb8f80);
  ball(157, 200);
}
