import { DownloadIcon } from "lucide-react";
import { LectureSelector } from "./lecture-selector";
import { Button } from "./ui/button";
import { VideoSelector } from "./video-selector";

export function DownloadForm(props: { department: string; code: string }) {
	return (
		<div className="flex flex-col gap-6">
			<LectureSelector department={props.department} code={props.code} />
			<VideoSelector />
			<Button size="lg">
				Download
				<DownloadIcon />
			</Button>
		</div>
	);
}
