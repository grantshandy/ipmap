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

export type LocationRecord = {
  ent: Entity;
  animIndex: number;
  direction: ConnectionDirection;
};

export const convertCoord = (crd: Coordinate): LonLat =>
  new LonLat(crd.lng, crd.lat);

export const directionColorString = (color: ConnectionDirection): string => {
  if (color == "up") {
    return "#c01c28";
  } else if (color == "down") {
    return "#26a269";
  } else {
    return "#cd9309";
  }
};

export const getPath = (
  ell: Ellipsoid,
  from: Coordinate,
  loc: CaptureLocation,
): ArcPath => {
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

  return Array.from({ length: ARC_POINTS }, (_, i) =>
    math.bezier3v(i / ARC_POINTS, startCart, p25Cart, p75Cart, endCart),
  ) as ArcPath;
};

const ARC_MIN_OPACITY = 0.25;
const ARC_MAX_OPACITY = 1.0;
const ARC_MIN_WEIGHT = 3;
const ARC_MAX_WEIGHT = 6;

export const calculateOpacity = (val: number, max: number) =>
  lerp(val, 0, max, ARC_MIN_OPACITY, ARC_MAX_OPACITY);

export const calculateWeight = (val: number, max: number) =>
  lerp(val, 0, max, ARC_MIN_WEIGHT, ARC_MAX_WEIGHT);
