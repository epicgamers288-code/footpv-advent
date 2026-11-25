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
  const open = (a: number, b: number) =>
    c.edges.some(
      ([i, j]: number[]) => (i === a && j === b) || (i === b && j === a),
    );
  g.lineStyle(3, 0xb7ebf0);
  for (let i = 0; i < 25; i++) {
    const x = (i % 5) * 60,
      y = 30 + Math.floor(i / 5) * 60;
    if (i % 5 === 4 || !open(i, i + 1))
      g.lineBetween(x + 60, y, x + 60, y + 60);
    if (!open(i, i + 5)) g.lineBetween(x, y + 60, x + 60, y + 60);
  }
  g.fillStyle(0xffcf6b);
  g.fillRect(254, 286, 32, 32);
  const pos = path.at(-1)!;
  player((pos % 5) * 60 + 30, 60 + Math.floor(pos / 5) * 60, 0x57e2cf);
}
