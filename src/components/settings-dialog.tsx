import { Dialog, DialogPortal } from "@radix-ui/react-dialog"
import { DialogContent, DialogHeader } from "./ui/dialog"
import { Settings } from "lucide-react"
import { Button } from "./ui/button";
import { useState } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { invoke } from "@tauri-apps/api/core";

enum Resolution {
    HighRes = "HighRes",
    LowRes = "LowRes",
}

type AppSettings = {
    resolution: Resolution,
}

export const SettingsDialog = () => {
    let [settings, setSettings] = useState<AppSettings>({ resolution: Resolution.HighRes });
    let [resolutionValue, setResolutionValue] = useState<Resolution>(settings.resolution);

    const [open, setOpen] = useState(false);
    const [cacheSize, setCacheSize] = useState("0.0G");

    async function computeCache() {
        try {
            setCacheSize(await invoke('get_cache_size'));
        } catch (e) { 
            console.error('Failed computing size!', e); 
            setCacheSize("Unknown")
        }
    }

    async function openSettings() {
        setOpen(true);
        try {
            let newSettings: AppSettings = await invoke('load_settings');
            setSettings(newSettings);
            setResolution(newSettings.resolution);
        } catch(e) {
            console.error('Failed to load old settings', e);
            // Save default settings if it does not already exist
            await saveSettings();
        }
        await computeCache();
    }

    async function clearCache() {
        await invoke('clear_cache');
        await computeCache();
    }

    async function saveSettings() {
        try {
            await invoke('save_settings', { settings })
        } catch(e) {
            console.error("Failed to save settings!", e);
        }
    }

    async function setResolution(value: Resolution) {
        settings.resolution = value;
        setResolutionValue(value);
        setSettings(settings);
    }

    return (
        <div>
            <Button onClick={openSettings} variant={"secondary"} size={"icon"} className="m-4 fixed bottom-0 right-0">
                <Settings />
            </Button>
            <Dialog open={open} onOpenChange={setOpen} >
                <DialogPortal>
                    <DialogContent>
                        <DialogHeader className="text-2xl font-bold">Settings</DialogHeader>
                        
                        {/* Video resolution */}
                        <div className="flex gap-4">
                            <div className="place-self-center text-nowrap">Video Quality</div>
                            {/* setting value here is kind of hacky, but it's the easiest thing I came up with
                                anyway, fix it for me pls (idk react)
                            */}
                            <SelectQuality onValueChange={setResolution} value={resolutionValue} /> 
                            <p className="text-xs place-self-center">Select the resolution you want to download<br />High Res: <b>720p</b>, Low Res: usually <b>480p</b></p>
                        </div>

                        {/* Clear cache */}
                        <div className="flex gap-4">
                            <Button variant={"destructive"} onClick={clearCache}>Clear Cache</Button>
                            <p className="text-xs place-self-center">Clears the cache in your temporary storage<br/>Currently your cache consumes <b>{cacheSize}</b> of storage</p>
                        </div>

                        <span style={ { gridTemplateColumns: "1fr 1fr" } } className="grid gap-4">
                            <Button onClick={() => { saveSettings(); setOpen(false); }}>Save</Button>
                            <Button onClick={() => setOpen(false)} variant={"destructive"}>Cancel</Button>
                        </span>
                    </DialogContent>
                </DialogPortal>
            </Dialog>
        </div>
    )
}

function SelectQuality({ ...props }: React.ComponentProps<typeof Select>) {
    return (
        <Select {...props} >
            <SelectTrigger className="text-nowrap w-42 h-10 select-none py-2 place-self-center border-2">
                <SelectValue placeholder="Select Quality" />
            </SelectTrigger>
            <SelectContent>
                <SelectItem value={ Resolution.HighRes }  className="py-2">
                    High Res
                </SelectItem>
                <SelectItem value={ Resolution.LowRes } className="py-2">
                    Low Res
                </SelectItem>
            </SelectContent>
        </Select>
    )
}