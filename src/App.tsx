import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";
import "./App.css";

function App() {
	const [greetMsg, setGreetMsg] = createSignal("");
	const [name, setName] = createSignal("");

	async function greet() {
		// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
		setGreetMsg(await invoke("greet", { name: name() }));
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
