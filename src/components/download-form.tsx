import { subjectAtom, videosAtom } from "@/lib/atoms";
import { logtoClient } from "@/lib/logto";
import { Channel, invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAtomValue } from "jotai";
import { BirdIcon, DownloadIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { LectureSelector } from "./lecture-selector";
import { SubjectSelector } from "./subject-selector";
import { Button } from "./ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogTitle,
} from "./ui/dialog";
import { Progress } from "./ui/progress";
import { MasterSelects, VideoSelector } from "./video-selector";

type DownloadProgressEvent = {
	percent: number;
};


type DownloadErrorEvent = {
	errors: string[];
};


const DownloadButton = () => {
	const videos = useAtomValue(videosAtom);
	const selectedVideos = useMemo(
		() => videos.filter((v) => v.selected),
		[videos],
	);
	const [open, setOpen] = useState(false);
	let [progressPercentage, setProgressPercentage] = useState(0);
	let [errors, setErrors] = useState<string[]>([]);
	let [complete, setComplete] = useState(false);

	const onProgress = new Channel<DownloadProgressEvent>();
	onProgress.onmessage = (message) => setProgressPercentage(message?.percent);

	const onError = new Channel<DownloadErrorEvent>();
	onError.onmessage = (message) => setErrors(prevErrors => [...prevErrors, ...message.errors]);

	async function handleClick() {
		setProgressPercentage(0);
		setErrors([]);
		setComplete(false);

		const baseFolder = await openDialog({
			directory: true,
			multiple: false,
		});
		if (!baseFolder) return;

		const token = await logtoClient.getIdToken();
		setOpen(true);
		// Use base folder instead of adding temp, since the temp file is chosen to be the default temp
		// file of the operating system.
		await invoke("download", { token, folder: baseFolder, videos: selectedVideos, onProgress, onError });
		setComplete(true);
	}

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<Button disabled={selectedVideos.length === 0} onClick={handleClick}>
				({selectedVideos.length}) Download
				<DownloadIcon />
			</Button>
			<DialogContent>
				<DialogTitle>Downloading Your Lectures...</DialogTitle>
				<DialogDescription>{progressPercentage}% Complete</DialogDescription>
				<Progress value={progressPercentage} />
				{errors.length > 0 ? <b>Errors:<br/></b> : <></> }
				
				{/* idk what to put for max height to make it look decent */}
				<div style={ { color: "red", overflow: "auto", maxHeight: "100px" } }>
					{errors.map((error, i) => (
						<span key={i}>
							{error}
							{i < errors.length - 1 && <br />}
						</span>
					))}
				</div>
				<Button onClick={() => setOpen(false)} disabled={!complete}>Ok</Button>
			</DialogContent>
		</Dialog>
	);
};

export const DownloadForm = () => {
	const subject = useAtomValue(subjectAtom);

	return (
		<div className="py-6">
			<SubjectSelector />
			{subject ? (
				<div className="flex flex-col">
					<div className="flex items-center gap-2 sticky top-0 py-6 bg-background">
						<LectureSelector />
						<MasterSelects />
						<DownloadButton />
					</div>
					<VideoSelector />
				</div>
			) : (
				<div className="flex flex-col gap-6 justify-center items-center py-12">
					<BirdIcon className="size-64 text-muted-foreground" />
					<p className="leading-7">no subject selected</p>
				</div>
			)}
		</div>
	);
};
