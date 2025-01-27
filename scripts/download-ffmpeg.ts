import { dirname, join } from "node:path";

/**
 * Collection of good ffmpeg static binaries
 */
const releases =
	"https://github.com/descriptinc/ffmpeg-ffprobe-static/releases/download/b6.1.2-rc.1";

/**
 * Mapping of binaries from release to where we are storing locally
 */
const binaries = [["linux-x64", "x86_64-unknown-linux-gnu"]];

/**
 * Location of the sidecar dir for tauri
 */
const sidecarDirectory = join(
	dirname(import.meta.dirname),
	"./src-tauri/binaries",
);

for (const [from, to] of binaries) {
	const target = join(sidecarDirectory, `ffmpeg-${to}`);
	if (await Bun.file(target).exists()) continue;
	const link = `${releases}/ffmpeg-${from}`;
	console.info(`Downloading ${link}`);
	const file = await fetch(link);
	await Bun.write(target, file);
	console.info(`Downloaded ${from} to ${target}`);
}
