let intervalId: Timer | null = null;

window.onload = () => {
	// avoid multiple interval instances
	if (intervalId !== null) {
		clearInterval(intervalId);
	}

	intervalId = setInterval(() => {
		const query = "img.ytmusic-player-bar";
		const thumbnailElement = document.querySelector(query);
		if (!thumbnailElement) {
			throw new Error(`Element not found with query: ${query}`);
		}
		const thumbnailUrl = thumbnailElement.getAttribute("src");

		try {
			fetch("http://localhost:8477", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					music_url: "thumbnail",
					thumbnail_url: thumbnailUrl,
				}),
			});
		} catch (e) {
			console.warn("Failed to send thumbnail to discord-sdk process", e);
		}
	}, 5000);
};
