import { invoke } from "@tauri-apps/api/core";
import { Settings } from "lucide-react";
import { useState } from "react";
import { Button } from "./ui/button";
import { Dialog, DialogPortal } from "./ui/dialog";
import { DialogContent, DialogTitle } from "./ui/dialog";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "./ui/select";
import Tooltip from "./ui/tooltip";
import { toast } from "sonner";

enum Resolution {
	HighRes = "HighRes",
	LowRes = "LowRes",
}

type AppSettings = {
	resolution: Resolution;
	base: string | null;
	format: string | null;
};

// Select remote automatically
const AUTO = "Auto";

export const SettingsDialog = () => {
	const [settings, setSettings] = useState<AppSettings>({
		resolution: Resolution.HighRes,
		base: null,
		format: null,
	});

	const [open, setOpen] = useState(false);
	const [cacheSize, setCacheSize] = useState("0.0KiB");

	async function computeCache() {
		try {
			setCacheSize(await invoke("get_cache_size"));
		} catch (e) {
			console.error("Failed computing size!", e);
			setCacheSize("Unknown");
		}
	}

	async function openSettings() {
		setOpen(true);
		try {
			const newSettings: AppSettings = await invoke("load_settings");
			setSettings(newSettings);
		} catch (e) {
			console.error("Failed to load old settings", e);
			// Save default settings if it does not already exist
			await saveSettings();
		}
		await computeCache();
	}

	async function clearCache() {
		await invoke("clear_cache");
		await computeCache();
	}

	async function saveSettings() {
		try {
			await invoke("save_settings", { settings });
			toast.success("Saved settings successfully!");
		} catch (e) {
			toast.error("Failed to save settings due to an error! Please try again later.");
			console.error("Failed to save settings!", e);
		}
	}

	async function saveClick() {
		let formats = ["{number}", "{date}"];

		if (settings.format != null && !formats.some(format => settings.format!.includes(format))) {
			toast.error("Format must include at least one of the following specifiers: " + formats.join(", "));
			setOpen(true);
			return;
		}
		
		await saveSettings();
		setOpen(false);
	}

	async function setResolution(value: Resolution) {
		setSettings((prev) => ({ ...prev, resolution: value }));
	}

	async function setFormat(value: string) {
		setSettings((prev) => ({ ...prev, format: (value.trim().length === 0 ? null : value) }));
	}

	async function setBase(value: string) {
		setSettings((prev) => ({
			...prev,
			base: (value == AUTO ? null : value),
		}));
	}

	return (
		<div>
			<Button
				onClick={openSettings}
				variant="secondary"
				size="icon"
				className="m-4 fixed bottom-0 right-0"
			>
				<Settings />
			</Button>
			<Dialog open={open} onOpenChange={setOpen}>
				<DialogPortal>
					<DialogContent className="p-0" aria-describedby={undefined}>
						<DialogTitle className="text-2xl font-bold px-6 mt-6">
							Settings
						</DialogTitle>
						<div className="flex flex-col gap-6 max-h-80 overflow-auto relative pb-2 px-6">
						{/* Video resolution */}
						<div className="flex flex-row items-center gap-4 justify-between">
							<div>
								<b>Video Quality</b>
								<p className="text-xs">
									Select download resolution
									<br />
									High Res: <b>720p</b>, Low Res: usually <b>480p</b>
								</p>
							</div>
							<SelectQuality
									onValueChange={setResolution}
									value={settings.resolution}
								/>
						</div>

						{/* Remote */}
						<div className="flex items-center gap-4 justify-between">
							<div>
								<b>Download Source</b>
								<p className="text-xs">
									Select source to download from
									<br />
									<b>Auto:</b>{" "}
									Try downloading from fastest source
									<br />
									<b>Remote:</b>{" "}
									Works anywhere, might be slower
									<br />
									<b>Local:</b>{" "}
									Works only on LAN and local WiFi, usually
									faster
								</p>
							</div>
							<SelectRemotes
								onValueChange={setBase}
								value={settings.base == null
									? AUTO
									: settings.base}
							/>
						</div>

						<div className="flex flex-col items-center gap-4">
							<div className="place-self-start">
								<b>Format</b>
								<p className="text-xs">
									Select the format you want the downloads to be in
									<br />
									Available format specifiers:  <span className="font-mono">
										<Tooltip content={"The name of the lecture"} ><span className="underline">{"{topic}"}</span></Tooltip>, <Tooltip content={"The lecture number"} ><span className="underline">{"{number}"}</span></Tooltip>, <Tooltip content={"The date when the lecture was taken"} ><span className="underline">{"{date}"}</span></Tooltip>, <Tooltip content={"The resolution of the lecture (high_res / low_res)"} ><span className="underline">{"{resolution}"}</span></Tooltip>
									</span>
									<br/>
										For example, you would use: <span className="font-mono">{"{number}_{topic}_{resolution}"}</span> to download in the default format
									<br />
									Keep empty to use the default download format
								</p>
							</div>
							<input type="text" placeholder="Enter format specifier" className="border-2 rounded py-2 px-3 outline-0 w-full text-sm" value={settings.format ?? ""}  onInput={(e) => setFormat(e.currentTarget.value)}/> 
						</div>

						{/* Clear cache */}
						<div className="flex gap-4">
							<Button
								variant={"destructive"}
								onClick={clearCache}
							>
								Clear Cache ({cacheSize})
							</Button>
							<p className="text-xs place-self-center">
								Clears the cache in your temporary storage
							</p>
						</div>
						</div>
						<div className="flex gap-4 w-full px-6 pb-6">
							<Button
								className="flex-1"
								onClick={() => saveClick()}
							>
								Save
							</Button>
							<Button
								className="flex-1"
								onClick={() => setOpen(false)}
								variant="destructive"
							>
								Cancel
							</Button>
						</div>
					</DialogContent>
				</DialogPortal>
			</Dialog>
		</div>
	);
};

function SelectQuality({ ...props }: React.ComponentProps<typeof Select>) {
	return (
		<Select {...props}>
			<SelectTrigger className="text-nowrap w-48 h-10 select-none py-2 place-self-center border-2 ">
				<SelectValue placeholder="Select Quality" />
			</SelectTrigger>
			<SelectContent>
				<SelectItem value={Resolution.HighRes} key={0} className="py-2">
					High Res
				</SelectItem>
				<SelectItem value={Resolution.LowRes} key={1} className="py-2">
					Low Res
				</SelectItem>
			</SelectContent>
		</Select>
	);
}

function SelectRemotes({ ...props }: React.ComponentProps<typeof Select>) {
	let bases: string[] = JSON.parse(import.meta.env.VITE_REMOTES);

	return (
		<Select {...props}>
			<SelectTrigger className="text-nowrap w-64 h-10 select-none py-2 place-self-center border-2">
				<SelectValue placeholder="Select Quality" />
			</SelectTrigger>
			<SelectContent>
				{bases.map((value, i) => (
					<SelectItem value={value} key={i} className="py-2">
						{/* Assume https:// is remote, otherwise all links are local */}
						{value.includes("https://")
							? "(Remote) " + value
							: "(Local) " + value}
					</SelectItem>
				))}
				<SelectItem value={AUTO} key={bases.length} className="py-2">
					Auto
				</SelectItem>
			</SelectContent>
		</Select>
	);
}
