import type {
  CaptureSession,
  ConnectionDirection,
  Coordinate,
} from "tauri-plugin-pcap-api";

export type PageView = "capture" | "search" | "trace";

class PageStateManager {
  page = $state<PageView>("search");
  globe = $state(false);

  constructor() {
    const stored = localStorage.getItem("pageState");

    if (stored) {
      const parsed = JSON.parse(stored);
      this.page = parsed.page;
      this.globe = parsed.globe;
    }

    $effect.root(() => {
      $effect(() => {
        localStorage.setItem(
          "pageState",
          JSON.stringify({ page: this.page, globe: this.globe }),
        );
      });
    });
  }
}

export const pageState = new PageStateManager();

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
