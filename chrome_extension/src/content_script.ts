// port.postMessage({ message: "ping", body: "hello from browser extension" });
setInterval(() => {
	const query = "img.ytmusic-player-bar";
	const thumbnailElement = document.querySelector(query);
	if (!thumbnailElement) {
		throw new Error(`Element not found with query: ${query}`);
	}
	const thumbnailUrl = thumbnailElement.getAttribute("src");
	console.log(thumbnailUrl);
	// send to local discord-sdk process
	fetch("http://localhost:8477", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			message: "thumbnail",
			body: thumbnailUrl,
		}),
	});
}, 5000);
