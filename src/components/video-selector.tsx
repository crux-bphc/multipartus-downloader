import { lectureAtom, videosAtom } from "@/lib/atoms";
import { fetchLex } from "@/lib/lex";
import { type PrimitiveAtom, useAtom, useAtomValue, useSetAtom } from "jotai";
import { splitAtom } from "jotai/utils";
import { useEffect } from "react";
import { Button } from "./ui/button";
import { Checkbox } from "./ui/checkbox";
import { Skeleton } from "./ui/skeleton";

const videoAtomsAtom = splitAtom(videosAtom, (item) => item.ttid);

const VideoItem = (props: { video: PrimitiveAtom<Multipartus.Video> }) => {
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
				className="flex justify-between py-3 flex-grow cursor-pointer"
			>
				<div className="inline-flex gap-2">
					<span className="bg-foreground text-primary px-1 rounded-sm text-bold">
						{video.number}
					</span>
					{video.topic}
				</div>
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
						number: videos.length - i,
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

export const MasterSelects = () => {
	const setVideos = useSetAtom(videosAtom);

	function selectAll() {
		setVideos((videos) =>
			videos.map((video) => ({ ...video, selected: true })),
		);
	}

	function deselectAll() {
		setVideos((videos) =>
			videos.map((video) => ({ ...video, selected: false })),
		);
	}

	return (
		<>
			<Button variant="secondary" onClick={deselectAll}>
				Deselect All
			</Button>
			<Button variant="secondary" onClick={selectAll}>
				Select All
			</Button>
		</>
	);
};
