import { invoke } from "@tauri-apps/api/core";
import { Settings } from "lucide-react";
import { useState } from "react";
import { Button } from "./ui/button";
import { Dialog, DialogPortal } from "./ui/dialog";
import { DialogContent, DialogHeader } from "./ui/dialog";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "./ui/select";

enum Resolution {
	HighRes = "HighRes",
	LowRes = "LowRes",
}

type AppSettings = {
	resolution: Resolution;
};

export const SettingsDialog = () => {
	const [settings, setSettings] = useState<AppSettings>({
		resolution: Resolution.HighRes,
	});

	const [open, setOpen] = useState(false);
	const [cacheSize, setCacheSize] = useState("0.0G");

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
		} catch (e) {
			console.error("Failed to save settings!", e);
		}
	}

	async function setResolution(value: Resolution) {
		setSettings((prev) => ({ ...prev, resolution: value }));
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
					<DialogContent className="gap-6">
						<DialogHeader className="text-2xl font-bold">Settings</DialogHeader>

						{/* Video resolution */}
						<div className="flex items-center gap-4">
							<div>
								<b>Video Quality</b>
								<p className="text-xs">
									Select the resolution you want to download
									<br />
									High Res: <b>720p</b>, Low Res: usually <b>480p</b>
								</p>
							</div>
							<SelectQuality
								onValueChange={setResolution}
								value={settings.resolution}
							/>
						</div>

						{/* Clear cache */}
						<div className="flex gap-4">
							<Button variant={"destructive"} onClick={clearCache}>
								Clear Cache ({cacheSize})
							</Button>
							<p className="text-xs place-self-center">
								Clears the cache in your temporary storage
							</p>
						</div>

						<div className="flex gap-4">
							<Button
								className="flex-1"
								onClick={() => {
									saveSettings();
									setOpen(false);
								}}
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
			<SelectTrigger className="text-nowrap w-42 h-10 select-none py-2 place-self-center border-2">
				<SelectValue placeholder="Select Quality" />
			</SelectTrigger>
			<SelectContent>
				<SelectItem value={Resolution.HighRes} className="py-2">
					High Res
				</SelectItem>
				<SelectItem value={Resolution.LowRes} className="py-2">
					Low Res
				</SelectItem>
			</SelectContent>
		</Select>
	);
}
