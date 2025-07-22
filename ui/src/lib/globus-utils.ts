import { Ellipsoid, Entity, LonLat, math, Vec3 } from "@openglobus/og";
import {
  lerp,
  type CaptureLocation,
  type ConnectionDirection,
  type Coordinate,
} from "./bindings";

type FixedSizeArray<N extends number, T> = Array<T> & { length: N };

export const ARC_POINTS = 70;

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

export const directionArcColors = (
  loc: CaptureLocation,
  max: number,
): ArcColors =>
  defaultArcColors([
    ...directionColor(loc.dir),
    calculateOpacity(loc.thr, max),
  ]);

export const getPath = (
  ell: Ellipsoid,
  from: Coordinate,
  loc: CaptureLocation,
  maxThroughput: number,
): {
  path: ArcPath;
  colors: ArcColors;
} => {
  const start = convertCoord(from);
  const end = convertCoord(loc.crd);

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

  const colors: ArcColors = directionArcColors(loc, maxThroughput);

  return { path, colors };
};

const ARC_MIN_OPACITY = 0;
const ARC_MAX_OPACITY = 1.0;
const ARC_MIN_WEIGHT = 3;
const ARC_MAX_WEIGHT = 6;

export const calculateOpacity = (val: number, max: number) =>
  lerp(val, 0, max, ARC_MIN_OPACITY, ARC_MAX_OPACITY);

export const calculateWeight = (val: number, max: number) =>
  lerp(val, 0, max, ARC_MIN_WEIGHT, ARC_MAX_WEIGHT);
