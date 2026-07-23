/** Pure SVG isometric helpers (2:1 projection). */

export interface Pt {
  X: number;
  Y: number;
}

/** Rzut izometryczny 2:1 zakotwiczony w (cx, cy). */
export function isoFactory(cx: number, cy: number, scale = 1) {
  return (x: number, y: number, z = 0): Pt => ({
    X: cx + (x - y) * scale,
    Y: cy + ((x + y) / 2) * scale - z * scale,
  });
}

/** Punkty do atrybutu <polygon points>. */
export const poly = (...pts: Pt[]): string =>
  pts.map((p) => `${p.X.toFixed(1)},${p.Y.toFixed(1)}`).join(" ");

/** Deterministyczny generator (kamienie w glebie zawsze takie same). */
export function seeded(seed: number): () => number {
  let s = seed;
  return () => {
    s = (s * 16807) % 2147483647;
    return s / 2147483647;
  };
}
