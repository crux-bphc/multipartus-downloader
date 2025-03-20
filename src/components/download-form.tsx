import { subjectAtom, videosAtom } from "@/lib/atoms";
import { logtoClient } from "@/lib/logto";
import { Channel, invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { logtoClient } from "@/lib/logto";
import { Channel, invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAtomValue } from "jotai";
import { BirdIcon, DownloadIcon } from "lucide-react";
import { useMemo, useState } from "react";
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
				<DialogDescription>{progressPercentage.toFixed(1)}% Complete</DialogDescription>
				<Progress value={progressPercentage} />
				{errors.length > 0 ? <b>Errors:<br/></b> : <></> }
				
				{/* idk what to put for max height to make it look decent */}
				<div className="whitespace-pre-wrap max-h-28 overflow-auto text-red-600">
					{errors.map((error, i) => (
						<span key={i}>
							{error}
							{i < errors.length - 1 && <br />}
						</span>
					))}
				</div>
				<span style={ { gridTemplateColumns: "1fr 1fr" } } className="grid gap-4">
					<Button onClick={() => setOpen(false)} disabled={!complete}>Ok</Button>
					<Button onClick={() => invoke('cancel_download')} disabled={complete} variant={"destructive"}>Cancel</Button>
				</span>
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
					<p className="leading-7">No subject selected</p>
					<p className="leading-7">No subject selected</p>
				</div>
			)}
		</div>
	);
};
