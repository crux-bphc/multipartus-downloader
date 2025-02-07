import { logtoClient } from "@/lib/logto";
import { onUrl, start } from "@fabianlars/tauri-plugin-oauth";

export default function HomePage() {
	async function handleLogin() {
		const port = await start();
		await logtoClient.signIn(`http://localhost:${port}`);

		await onUrl(async (url) => {
			await logtoClient.handleSignInCallback(url);
			if (await logtoClient.isAuthenticated()) {
				alert("Authentication successful!");
				console.log(await logtoClient.getIdToken());
			} else {
				alert("Authentication failed!");
			}
		});
	}

	return (
		<main class="container">
			<button onClick={handleLogin} type="button">
				Login
			</button>
		</main>
	);
}
