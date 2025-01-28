import { Command } from "@tauri-apps/plugin-shell";
import { createSignal } from "solid-js";

export default function HomePage() {
	const [greetMsg, setGreetMsg] = createSignal("");

	async function greet() {
		const command = Command.sidecar("binaries/ffmpeg", ["-version"]);
		const output = await command.execute();
		setGreetMsg(output.stdout);
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
