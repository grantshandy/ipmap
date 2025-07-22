import { divIcon, geodesic, marker, type Marker } from "leaflet";
import {
  CAPTURE_COLORS,
  CAPTURE_VARY_SIZE,
  lerp,
  type CaptureLocation,
  type Coordinate,
} from "./bindings";
import { GeodesicLine } from "leaflet.geodesic";

const ARC_MIN_OPACITY = 0.25;
const ARC_MAX_OPACITY = 1.0;
const ARC_MIN_WEIGHT = 1.5;
const ARC_MAX_WEIGHT = 6;

export const newArc = (
  from: Coordinate,
  record: CaptureLocation,
  maxThroughput: number,
): GeodesicLine => {
  const { weight, opacity } = CAPTURE_VARY_SIZE
    ? calculateWeights(record.thr, maxThroughput)
    : { weight: 2, opacity: 0.8 };

  return geodesic([from, record.crd], {
    steps: 5,
    // TODO: default styles other than basic blue?
    className: CAPTURE_COLORS ? record.dir : "",
    weight,
    opacity,
  });
};

export const updateArc = (
  record: CaptureLocation,
  arc: GeodesicLine,
  maxThroughput: number,
) => {
  if (CAPTURE_COLORS) {
    const svgElement = arc.getElement();
    if (svgElement) {
      svgElement.setAttribute("class", `leaflet-interactive ${record.dir}`);
    }
  }

  if (CAPTURE_VARY_SIZE) {
    arc.setStyle(calculateWeights(record.thr, maxThroughput));
  }
};

export const calculateWeights = (
  throughput: number,
  maxThroughput: number,
) => ({
  weight: lerp(throughput, 0, maxThroughput, ARC_MIN_WEIGHT, ARC_MAX_WEIGHT),
  opacity: lerp(throughput, 0, maxThroughput, ARC_MIN_OPACITY, ARC_MAX_OPACITY),
});

export const newMarker = (loc: CaptureLocation): Marker =>
  marker(loc.crd, { icon: markerIcon(loc, false) });

export const markerIcon = (loc: CaptureLocation, focused: boolean) => {
  const iconSize = focused ? 30 : 20;
  const iconAnchor = iconSize / 2;

  return divIcon({
    html: `<span>${Object.keys(loc.ips).length}</span>`,
    className: focused ? "marker-icon-active" : "marker-icon",
    iconSize: [iconSize, iconSize],
    iconAnchor: [iconAnchor, iconAnchor],
  });
};
