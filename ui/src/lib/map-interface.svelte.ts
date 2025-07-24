import type { Snippet } from "svelte";
import type {
  CaptureSession,
  ConnectionDirection,
  Coordinate,
} from "./bindings";

export interface MapArgs {
  capture?: CaptureSession | null;
  focused?: string | null;
  children?: Snippet;
}

export interface MapInterface {
  createMarker(key: string, crd: Coordinate, count: number): void;
  updateMarker(key: string, crd: Coordinate, count: number): void;
  removeMarker(key: string): void;
  createArc(
    key: string,
    from: Coordinate,
    to: Coordinate,
    thr: number,
    dir: ConnectionDirection,
  ): void;
  updateArc(
    key: string,
    from: Coordinate,
    to: Coordinate,
    thr: number,
    dir: ConnectionDirection,
  ): void;
  removeArc(key: string): void;

  /** Zoom from 0 to 1 */
  flyToPoint(crd: Coordinate, zoom: number): void;
}
