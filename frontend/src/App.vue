<template>
  <div id="mapid" class="container"></div>
</template>

<style>
.container {
    position:fixed;
    padding:0;
    margin:0;
    top:0;
    left:0;
    width: 100%;
    height: 100%;
}

body {
    background: #343434;
    color: #ffffff;
}
</style>

<script>
import { map, tileLayer, marker } from "leaflet";

export default {
	name: "App",
  data() {
    return {
      eventSource: new EventSource("/ip_stream"),
      ipIndex: new Array(),
    }
  },
  mounted() {
    let myMap = map('mapid').setView([20, 0], 2);

    tileLayer('https://api.mapbox.com/styles/v1/{id}/tiles/{z}/{x}/{y}?access_token=pk.eyJ1IjoibWFwYm94IiwiYSI6ImNpejY4NXVycTA2emYycXBndHRqcmZ3N3gifQ.rJcFIG214AriISLbB6B5aw', {
        maxZoom: 18,
        id: 'mapbox/streets-v11',
        tileSize: 512,
        zoomOffset: -1
    }).addTo(myMap);

    this.eventSource.onmessage = (event) => {
      let obj = JSON.parse(event.data);
      
      if (!this.ipIndex.includes(obj.ip)) {
        this.ipIndex.push(obj.ip);
        
        console.log(obj.ip);
        
        marker([obj.latitude, obj.longitude]).addTo(myMap).bindPopup(`<b>${obj.ip}</b> - ${obj.city}\n${obj.org}`);
      }
    };
  },
}
</script>