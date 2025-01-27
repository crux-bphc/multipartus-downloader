import { createSignal } from "solid-js";
import "./App.css";
import { Command } from "@tauri-apps/plugin-shell";

function App() {
	const [greetMsg, setGreetMsg] = createSignal("");

	async function greet() {
		const command = Command.sidecar("binaries/ffmpeg");
		const output = await command.execute();
		console.log(output);
		setGreetMsg("Clicked");
	}

	return (
		<main class="container">
			<button onClick={greet} type="button">
				Greet
			</button>
			<p>{greetMsg()}</p>
		</main>
	);
}

export default App;
