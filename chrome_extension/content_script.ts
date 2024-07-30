setInterval(() => {
	const query = "img .image .ytmusic-player-bar";
	const thumbnailElement = document.querySelector(query);
	if (!thumbnailElement) {
		throw new Error(`Element not found: ${query}`);
	}
	const thumbnailUrl = thumbnailElement.getAttribute("src");
	// send to local discord-sdk process
	// TODO: websocket?
	fetch("http://127:0.0.1:8477", {
		method: "POST",
		body: JSON.stringify({ thumbnailUrl }),
	});
}, 5000);
