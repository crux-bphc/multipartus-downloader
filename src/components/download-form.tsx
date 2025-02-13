import { atom, useAtomValue } from "jotai";
import { BirdIcon, DownloadIcon } from "lucide-react";
import { LectureSelector } from "./lecture-selector";
import { Button } from "./ui/button";
import { VideoSelector } from "./video-selector";

// selected subject
export const subjectAtom = atom<[string, string]>();

// selected lecture section for the selected subject
export const lectureAtom = atom<[number, number]>();

// selected video ttids for selected lecture
// -1 if video is not selected, {ttid} if selected
export const ttidsAtom = atom<number[]>([]);

export function DownloadForm() {
	const subject = useAtomValue(subjectAtom);

	return (
		<div>
			{subject ? (
				<div className="flex flex-col gap-6">
					<LectureSelector />
					<VideoSelector />
					<Button size="lg">
						Download
						<DownloadIcon />
					</Button>
				</div>
			) : (
				<div className="flex flex-col gap-6 justify-center items-center py-12">
					<BirdIcon className="w-64 h-64 text-muted-foreground" />
					<p className="leading-7">no subject selected</p>
				</div>
			)}
		</div>
	);
}
