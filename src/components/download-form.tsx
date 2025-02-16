import { subjectAtom } from "@/lib/atoms";
import { useAtomValue } from "jotai";
import { BirdIcon, DownloadIcon } from "lucide-react";
import { LectureSelector } from "./lecture-selector";
import { Button } from "./ui/button";
import { VideoSelector } from "./video-selector";

export function DownloadForm() {
	const subject = useAtomValue(subjectAtom);

	return (
		<div>
			{subject ? (
				<div className="flex flex-col">
					<div className="flex items-center gap-3 sticky top-0 py-6 bg-card">
						<LectureSelector />
						<Button>
							Download
							<DownloadIcon />
						</Button>
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
}
