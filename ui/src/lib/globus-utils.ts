import { Ellipsoid, Entity, LonLat, math, Vec3 } from "@openglobus/og";
import type { ConnectionDirection, Coordinate } from "./bindings";

type FixedSizeArray<N extends number, T> = Array<T> & { length: N };

export const ARC_POINTS = 70;
const DEFAULT_OPACITY = 0.3;

export type ArcPath = FixedSizeArray<typeof ARC_POINTS, Vec3>;
export type ArcColors = FixedSizeArray<typeof ARC_POINTS, NumberArray4>;

type NumberArray3 = [number, number, number];
type NumberArray4 = [number, number, number, number];

export type LocationRecord = {
  ent: Entity;
  animIndex: number;
  colors: ArcColors;
  direction: ConnectionDirection;
};

export const convertCoord = (crd: Coordinate): LonLat =>
  new LonLat(crd.lng, crd.lat);

const UP_COLOR: NumberArray3 = [38 / 255, 162 / 255, 105 / 255]; // Green
const DOWN_COLOR: NumberArray3 = [192 / 255, 28 / 255, 40 / 255]; // Red
const MIXED_COLOR: NumberArray3 = [205 / 255, 147 / 255, 9 / 255]; // Yellow/Gold

const directionColor = (color: ConnectionDirection): NumberArray3 => {
  if (color == "up") {
    return UP_COLOR;
  } else if (color == "down") {
    return DOWN_COLOR;
  } else {
    return MIXED_COLOR;
  }
};

export const defaultArcColors = (color: NumberArray4): ArcColors =>
  Array.from({ length: ARC_POINTS }, () => color) as ArcColors;

export const directionArcColors = (color: ConnectionDirection): ArcColors =>
  defaultArcColors([...directionColor(color), DEFAULT_OPACITY]);

export const getPath = (
  ell: Ellipsoid,
  from: Coordinate,
  to: Coordinate,
  direction: ConnectionDirection,
): {
  path: ArcPath;
  colors: ArcColors;
} => {
  const start = convertCoord(from);
  const end = convertCoord(to);

  const { distance, initialAzimuth } = ell.inverse(start, end);

  let p25 = ell.getGreatCircleDestination(
      start,
      initialAzimuth,
      distance * 0.25,
    ),
    p75 = ell.getGreatCircleDestination(start, initialAzimuth, distance * 0.75);

  start.height = 50;
  end.height = 50;
  const h = distance / 4;
  p25.height = h;
  p75.height = h;

  const startCart = ell.lonLatToCartesian(start),
    endCart = ell.lonLatToCartesian(end),
    p25Cart = ell.lonLatToCartesian(p25),
    p75Cart = ell.lonLatToCartesian(p75);

  const path: ArcPath = Array.from({ length: ARC_POINTS }, (_, i) =>
    math.bezier3v(i / ARC_POINTS, startCart, p25Cart, p75Cart, endCart),
  ) as ArcPath;

  const colors: ArcColors = directionArcColors(direction);

  return { path, colors };
};

export const animateLine = (_loc: LocationRecord) => {
  // const polyline = loc.ent.polyline;
  // if (!polyline) return;
  // // Increment and wrap the animation index
  // loc.animIndex++;
  // if (loc.animIndex > ARC_POINTS + 4) {
  //   // Add padding for the tail
  //   loc.animIndex = 0;
  // }
  // const ind = loc.animIndex;
  // const [r, g, b] = loc.baseColor;
  // const pathIndex = 0; // Each entity has only one path
  // // Set the "glow" head and tail with decreasing alpha
  // polyline.setPointColor([r, g, b, 0.8], ind, pathIndex);
  // polyline.setPointColor([r, g, b, 0.6], ind - 1, pathIndex);
  // polyline.setPointColor([r, g, b, 0.3], ind - 2, pathIndex);
  // polyline.setPointColor([r, g, b, 0.1], ind - 3, pathIndex);
  // // Reset the color of the point that the tail just passed
  // const resetIndex = ind - 4;
  // const originalColor = loc.originalColors[resetIndex] || loc.originalColors[0];
  // polyline.setPointColor(originalColor, resetIndex, pathIndex);
};
