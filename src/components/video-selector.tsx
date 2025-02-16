import { type Video, lectureAtom, videosAtom } from "@/lib/atoms";
import { fetchLex } from "@/lib/lex";
import { type PrimitiveAtom, useAtom, useAtomValue, useSetAtom } from "jotai";
import { splitAtom } from "jotai/utils";
import { useEffect } from "react";
import { Checkbox } from "./ui/checkbox";
import { Skeleton } from "./ui/skeleton";

const videoAtomsAtom = splitAtom(videosAtom, (item) => item.ttid);

const VideoItem = (props: { video: PrimitiveAtom<Video> }) => {
	const formatter = new Intl.DateTimeFormat("en-US");
	const [video, setVideo] = useAtom(props.video);

	return (
		<div className="flex items-center gap-3 px-3 bg-card rounded-lg hover:shadow-sm transition-shadow border">
			<Checkbox
				id={`ttid-${video.ttid}`}
				checked={video.selected}
				onCheckedChange={(checked) => {
					setVideo((prev) => ({ ...prev, selected: !!checked }));
				}}
			/>
			<label
				htmlFor={`ttid-${video.ttid}`}
				className="flex flex-col py-3 flex-grow cursor-pointer"
			>
				<span>
					<span className="bg-foreground text-background px-1 rounded-sm mr-1 text-bold">
						{video.index}
					</span>
					{video.topic}
				</span>
				<span className="text-sm text-muted-foreground">
					{formatter.format(new Date(video.startTime))}
				</span>
			</label>
		</div>
	);
};

export const VideoSelector = () => {
	const setVideo = useSetAtom(videosAtom);
	const lecture = useAtomValue(lectureAtom);
	const videoAtoms = useAtomValue(videoAtomsAtom);

	useEffect(() => {
		if (lecture) {
			setVideo([]);
			// fetch new videos when lecture changes
			fetchLex<Multipartus.Video[]>(`lecture/${lecture.join("/")}`)
				.then((videos) =>
					videos.map((video, i) => ({
						...video,
						selected: true,
						index: videos.length - i,
					})),
				)
				.then(setVideo);
		}
	}, [lecture]);

	if (videoAtoms.length === 0) {
		return <Skeleton className="h-16" />;
	}

	return (
		<div className="flex flex-col gap-3">
			{videoAtoms.map((video, i) => (
				// biome-ignore lint/suspicious/noArrayIndexKey: <explanation>
				<VideoItem key={i} video={video} />
			))}
		</div>
	);
};
