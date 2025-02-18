import { $ } from "bun";
import { dirname, join } from "node:path";
import { mkdir } from "node:fs/promises";

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
await mkdir(sidecarDirectory, { recursive: true });

async function download(targetTriple: keyof typeof binaries) {
	const target = join(sidecarDirectory, `ffmpeg-${targetTriple}`);
	if (await Bun.file(target).exists()) return;

	try {
		await $`curl -L -o ${target} ${binaries[targetTriple]}`;
	} catch {
		console.error("Make sure you have curl installed on your shell");
	}

	if (targetTriple.includes("linux")) {
		await $`chmod +x ${target}`;
	}

	console.info(`Downloaded ${binaries[targetTriple]} to ${target}`);
}

switch (process.platform) {
	case "linux":
		download("x86_64-unknown-linux-gnu");
		break;
	case "win32":
		download("x86_64-pc-windows-msvc.exe");
		break;
}
