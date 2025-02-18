import { subjectAtom, videosAtom } from "@/lib/atoms";
import { useAtomValue } from "jotai";
import { BirdIcon, DownloadIcon } from "lucide-react";
import { useMemo } from "react";
import { LectureSelector } from "./lecture-selector";
import { SubjectSelector } from "./subject-selector";
import { Button } from "./ui/button";
import { MasterSelects, VideoSelector } from "./video-selector";
import { invoke } from '@tauri-apps/api/core';
import { logtoClient } from "@/lib/logto";
import { open } from '@tauri-apps/plugin-dialog';
import { join } from '@tauri-apps/api/path';

const DownloadButton = () => {
	const videos = useAtomValue(videosAtom);
	const selectedVideos = useMemo(
		() => videos.filter((v) => v.selected),
		[videos],
	);

	async function handleClick() {
		const baseFolder = await open({
			directory: true,
			multiple: false
		});
		if (!baseFolder) return;

		const folder = await join(baseFolder, 'BITS TEMP');


		const token = await logtoClient.getIdToken();
		await invoke('download', { token, folder, videos: selectedVideos });
	}

	return (
		<Button disabled={selectedVideos.length === 0} onClick={handleClick}>
			({selectedVideos.length}) Download
			<DownloadIcon />
		</Button>
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
