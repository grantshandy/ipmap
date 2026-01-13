import type { Snippet } from "svelte";
import type {
  CaptureSession,
  ConnectionDirection,
  Coordinate,
} from "tauri-plugin-pcap-api";

export const pageState: {
  page: "capture" | "search" | "trace";
  globe: boolean;
} = $state({ page: "search", globe: false });

export interface MapArgs {
  capture?: CaptureSession | null;
  focused?: string | null;
}

export interface MapComponent {
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

  zoomIn(): void;
  zoomOut(): void;
}
