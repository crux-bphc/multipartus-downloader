import { $ } from "bun";
import { dirname, join } from "node:path";

/**
 * Mapping of binaries from release to where we are storing locally
 */
const binaries = {
	"x86_64-unknown-linux-gnu":
		"https://github.com/eugeneware/ffmpeg-static/releases/download/b6.0/ffmpeg-linux-x64",
	"x86_64-pc-windows-msvc.exe":
		"https://github.com/eugeneware/ffmpeg-static/releases/download/b6.0/ffmpeg-win32-x64",
};

/**
 * Location of the sidecar dir for tauri
 */
const sidecarDirectory = join(
	dirname(import.meta.dirname),
	"./src-tauri/binaries",
);

for await (const [to, from] of Object.entries(binaries)) {
	const target = join(sidecarDirectory, `ffmpeg-${to}`);
	if (await Bun.file(target).exists()) continue;

	try {
		await $`curl -L -o ${target} ${from}`;
	} catch {
		console.error("Make sure you have curl installed on your shell");
	}

	if (to.includes("linux")) {
		await $`chmod +x ${target}`;
	}

	console.info(`Downloaded ${from} to ${target}`);
}
