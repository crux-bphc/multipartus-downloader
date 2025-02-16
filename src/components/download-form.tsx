import { subjectAtom, videosAtom } from "@/lib/atoms";
import { useAtomValue } from "jotai";
import { BirdIcon, DownloadIcon } from "lucide-react";
import { LectureSelector } from "./lecture-selector";
import { Button } from "./ui/button";
import { MasterSelects, VideoSelector } from "./video-selector";
import { useMemo } from "react";

const DownloadButton = () => {
	const videos = useAtomValue(videosAtom);
	const selectCount = useMemo(
		() => videos.filter((v) => v.selected).length,
		[videos],
	);
	return (
		<Button disabled={selectCount === 0}>
			({selectCount}) Download
			<DownloadIcon />
		</Button>
	);
};

export const DownloadForm = () => {
	const subject = useAtomValue(subjectAtom);

	return (
		<div>
			{subject ? (
				<div className="flex flex-col">
					<div className="flex items-center gap-3 sticky top-0 py-6 bg-card">
						<LectureSelector />
						<MasterSelects />
						<DownloadButton />
					</div>
					<VideoSelector />
				</div>
			) : (
				<div className="flex flex-col gap-6 justify-center items-center py-12">
					<BirdIcon className="w-64 h-64 text-muted-foreground" />
					<p className="leading-7">no subject selected</p>
				</div>
			)}
		</div>
	);
};
