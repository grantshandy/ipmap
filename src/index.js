document.addEventListener("keypress", function onEvent(event) {
    if (event.key === "f") {
		window.webkit.messageHandlers.external.postMessage('toggleFullscreen');
	} else if (event.key = "Escape") {
		window.webkit.messageHandlers.external.postMessage('exitFullscreen');
	}
	
	if (event.key == "q") {
		window.webkit.messageHandlers.external.postMessage('quit');
	}

	if (event.key == "c") {
		window.webkit.messageHandlers.external.postMessage('credits');
	}
});

window.open("https://google.com");

var mymap = L.map('mapid').setView([20, 0], 2);

L.tileLayer('https://api.mapbox.com/styles/v1/{id}/tiles/{z}/{x}/{y}?access_token=pk.eyJ1IjoibWFwYm94IiwiYSI6ImNpejY4NXVycTA2emYycXBndHRqcmZ3N3gifQ.rJcFIG214AriISLbB6B5aw', {
    maxZoom: 18,
    id: 'mapbox/streets-v11',
    tileSize: 512,
    zoomOffset: -1
}).addTo(mymap);

var alreadyAdded = new Set();
var cityList = new Set ();

function addMarkers(jsonText) {
	for(var i = 0; i < jsonText.length; i++) {
		var obj = jsonText[i];

		if (alreadyAdded.has(obj.ip)) {
			continue;
		}

		cityList.add(obj.city);

		L.marker([obj.lat, obj.lon]).addTo(mymap).bindPopup("<b>" + obj.ip + "</b> - " + obj.city);
		alreadyAdded.add(obj.ip);
	}
}

(function loop() {
	setTimeout(function () {
		window.webkit.messageHandlers.external.postMessage('requestData');
		loop();
	}, 1000);
}());