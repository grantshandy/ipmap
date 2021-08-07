var mymap = L.map('mapid').setView([20, 0], 2);

L.tileLayer('https://api.mapbox.com/styles/v1/{id}/tiles/{z}/{x}/{y}?access_token=pk.eyJ1IjoibWFwYm94IiwiYSI6ImNpejY4NXVycTA2emYycXBndHRqcmZ3N3gifQ.rJcFIG214AriISLbB6B5aw', {
    maxZoom: 18,
    attribution: 'Map data &copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors, ' +
        'Imagery © <a href="https://www.mapbox.com/">Mapbox</a>',
    id: 'mapbox/streets-v11',
    tileSize: 512,
    zoomOffset: -1
}).addTo(mymap);

var alreadyAdded = new Set();

function addMarkers(jsonText) {
	for(var i = 0; i < jsonText.length; i++) {
		var obj = jsonText[i];

		if (alreadyAdded.has(obj.ip)) {
			continue;
		}

		document.getElementById("totalIps").innerHTML = jsonText.length + " Unique IP's";

		L.marker([obj.lat, obj.lon]).addTo(mymap).bindPopup("<b>" + obj.ip + "</b> - " + obj.city);
		alreadyAdded.add(obj.ip);
	}
}

// function externTest {
	
// }

(function loop() {
	setTimeout(function () {
		window.webkit.messageHandlers.external.postMessage('rustFunc');
		loop();
	}, 1000);
}());