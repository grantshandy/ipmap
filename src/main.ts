import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";

invoke("start_polling");

import Map from "ol/Map";
import OSM from "ol/source/OSM";
import TileLayer from "ol/layer/Tile";
import View from "ol/View";
import Zoom from "ol/control/Zoom";

new Map({
  target: "map",
  controls: [new Zoom()],
  layers: [
    new TileLayer({
      source: new OSM(),
    }),
  ],
  view: new View({
    center: [0, 0],
    zoom: 2,
  }),
});

interface Connection {
  city: string;
  lat: number;
  lon: number;
  ip: string;
}

listen("connection", (event: any) => {
  if (event.payload) {
    let connection: Connection = event.payload;

    console.log(connection);
  }
});
