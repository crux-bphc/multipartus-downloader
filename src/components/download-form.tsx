import { subjectAtom, videosAtom } from "@/lib/atoms";
import { logtoClient } from "@/lib/logto";
import { Channel, invoke } from "@tauri-apps/api/core";
import { join } from "@tauri-apps/api/path";
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
	data: string;
};

const DownloadButton = () => {
	const videos = useAtomValue(videosAtom);
	const selectedVideos = useMemo(
		() => videos.filter((v) => v.selected),
		[videos],
	);
	const [open, setOpen] = useState(false);

	const onProgress = new Channel<DownloadProgressEvent>();
	onProgress.onmessage = () => {};

	async function handleClick() {
		const baseFolder = await openDialog({
			directory: true,
			multiple: false,
		});
		if (!baseFolder) return;

		const folder = await join(baseFolder, "BITS TEMP");

		const token = await logtoClient.getIdToken();
		setOpen(true);
		await invoke("download", { token, folder, videos: selectedVideos });
		setOpen(false);
	}

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<Button disabled={selectedVideos.length === 0} onClick={handleClick}>
				({selectedVideos.length}) Download
				<DownloadIcon />
			</Button>
			<DialogContent>
				<DialogTitle>Download Progress</DialogTitle>
				<DialogDescription>TODO</DialogDescription>
				<Progress value={50} />
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
