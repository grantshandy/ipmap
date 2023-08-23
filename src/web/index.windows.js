var mymap = L.map('mapid').setView([20, 0], 2);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
    maxZoom: 18,
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

		L.marker([obj.lat, obj.lon]).addTo(mymap).bindPopup("<b>" + obj.ip + "</b> - " + obj.city);
		alreadyAdded.add(obj.ip);
	}
}

(function loop() {
	setTimeout(function () {
		external.invoke('requestData');
		loop();
	}, 1000);
}());

document.addEventListener("keypress", function onEvent(event) {
	switch(event.key) {
		case "f":
			external.invoke('toggleFullscreen');
			break;
		case "q":
			external.invoke('quit');
			break;
		case "c":
			external.invoke('credits');
			break;
		case "Escape":
			external.invoke('exitFullscreen');
			break;
	}
});