import type Phaser from "phaser";
export type Frame = {
  g: Phaser.GameObjects.Graphics;
  c: any;
  t: number;
  phase: number;
  path: number[];
  ball: (x: number, y: number) => void;
  player: (x: number, y: number, col: number) => void;
  glide: { x: number; y: number; f: number } | null;
};
