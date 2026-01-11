import { Ellipsoid, Entity, LonLat, math, Vec3 } from "@openglobus/og";
import type { ConnectionDirection, Coordinate } from "tauri-plugin-pcap-api";

type FixedSizeArray<N extends number, T> = Array<T> & { length: N };

export const ARC_POINTS = 70;
export const DASH_TO_GAP_RATIO = 1.5; // dash is 1.5 times the length of the gap
export const NUMBER_OF_DASHES = 10;
export const DASH_LENGTH_POINTS =
  ARC_POINTS / (NUMBER_OF_DASHES * (1 + 1 / DASH_TO_GAP_RATIO));
export const OSCILATION_RANGE =
  DASH_LENGTH_POINTS + DASH_LENGTH_POINTS / DASH_TO_GAP_RATIO;

export type ArcPath = FixedSizeArray<typeof ARC_POINTS, Vec3>;
export type DashedPaths = Vec3[][];

export type ArcRecord = {
  arc: Entity;
  animIndex: number;
  fullPath: ArcPath;
  direction: ConnectionDirection;
};

export const getPath = (
  ell: Ellipsoid,
  from: Coordinate,
  to: Coordinate,
): ArcPath => {
  const start = new LonLat(from.lng, from.lat);
  const end = new LonLat(to.lng, to.lat);

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

  return Array.from({ length: ARC_POINTS + 1 }, (_, i) =>
    math.bezier3v(i / ARC_POINTS, startCart, p25Cart, p75Cart, endCart),
  ) as ArcPath;
};

export const getDashedPath = (
  fullPath: ArcPath,
  numDashes: number,
  dashToGapRatio: number,
  offset: number,
): DashedPaths => {
  const totalPoints = fullPath.length;

  const dashLengthPoints = totalPoints / (numDashes * (1 + 1 / dashToGapRatio));
  const gapLengthPoints = dashLengthPoints / dashToGapRatio;
  const cycleLengthPoints = dashLengthPoints + gapLengthPoints;

  const paths: DashedPaths = [];

  // Normalize the offset to be within one full cycle (from 0 to cycleLengthPoints)
  let normalizedOffset = offset % cycleLengthPoints;
  if (normalizedOffset < 0) {
    normalizedOffset += cycleLengthPoints;
  }

  for (let i = 0; i < numDashes; i++) {
    const startOffset = i * cycleLengthPoints + normalizedOffset;
    const endOffset = startOffset + dashLengthPoints;

    // Create a new array for the current dash segment
    const path: Vec3[] = [];

    // Iterate through the full path to extract the points for the current dash
    for (let j = Math.floor(startOffset); j < Math.ceil(endOffset); j++) {
      // Use the modulo operator to handle wrapping around the end of the path
      const index = j % totalPoints;
      path.push(fullPath[index]);
    }

    // Ensure the path has at least two points to form a valid line segment
    if (path.length >= 2) {
      paths.push(path);
    }
  }
  return paths;
};
