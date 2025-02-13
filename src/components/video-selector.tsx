import { fetchLex } from "@/lib/lex";
import { atom, useAtom, useAtomValue } from "jotai";
import { loadable } from "jotai/utils";
import { useEffect, useMemo } from "react";
import { lectureAtom, ttidsAtom } from "./download-form";
import { Button } from "./ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardFooter,
	CardHeader,
	CardTitle,
} from "./ui/card";
import { Checkbox } from "./ui/checkbox";
import { ScrollArea } from "./ui/scroll-area";
import { Skeleton } from "./ui/skeleton";

const videosAtom = loadable(
	atom(async (get, { signal }) => {
		const lecture = get(lectureAtom);
		if (!lecture) {
			return [];
		}

		const videos = await fetchLex<Multipartus.Video[]>(
			`lecture/${lecture.join("/")}`,
			{ signal },
		);

		return videos;
	}),
);

const Videos = () => {
	const videos = useAtomValue(videosAtom);
	const [ttids, setTTIDs] = useAtom(ttidsAtom);

	useEffect(() => {
		if (videos.state === "hasData") {
			setTTIDs(videos.data.map((video) => video.ttid));
		}
	}, [videos]);

	if (videos.state !== "hasData") {
		return <Skeleton className="h-full" />;
	}

	return (
		<div className="grid grid-cols-3 gap-2">
			{videos.data.map((video, i) => (
				<div
					key={video.ttid}
					className="flex flex-col justify-around rounded-lg border"
				>
					<img
						className="w-full rounded-t-[inherit]"
						src={`https://bitshyd.impartus.com/download1/embedded/thumbnails/${video.ttid}.jpg`}
						alt={video.topic}
					/>
					<div className="p-2">
						<div className="flex items-center justify-between gap-2">
							<label
								htmlFor={`video-${video.ttid}`}
								className="flex items-center gap-2 text-lg truncate text-ellipsis"
							>
								<span className="bg-primary text-primary-foreground px-1 rounded">
									{videos.data.length - i}
								</span>
								{video.topic}
							</label>
							<Checkbox
								checked={ttids[i] === video.ttid}
								onCheckedChange={(checked) =>
									setTTIDs((ttids) =>
										ttids.map((ttid, j) => {
											if (i === j) {
												return checked ? video.ttid : -1;
											}
											return ttid;
										}),
									)
								}
								id={`video-${video.ttid}`}
							/>
						</div>
					</div>
				</div>
			))}
		</div>
	);
};

const SelectorFooter = () => {
	const videos = useAtomValue(videosAtom);
	const [ttids, setTTIDs] = useAtom(ttidsAtom);

	const selectedVideosCount = useMemo(
		() => ttids.reduce((acc, i) => (i === -1 ? acc : acc + 1), 0),
		[ttids],
	);

	if (videos.state !== "hasData") {
		return <Skeleton className="h-8" />;
	}

	return (
		<>
			<span className="text-muted-foreground text-bold mr-auto">
				({selectedVideosCount}) Selected
			</span>
			<Button
				type="button"
				variant="secondary"
				onClick={() => setTTIDs((ttids) => ttids.map(() => -1))}
			>
				Deselect All
			</Button>
			<Button
				type="button"
				onClick={() => setTTIDs(videos.data.map(({ ttid }) => ttid))}
			>
				Select All
			</Button>
		</>
	);
};

export const VideoSelector = () => {
	return (
		<Card>
			<CardHeader>
				<CardTitle>Select to Download</CardTitle>
				<CardDescription>
					Choose the videos you want to download.
				</CardDescription>
			</CardHeader>
			<CardContent>
				<ScrollArea className="h-86 pr-4">
					<Videos />
				</ScrollArea>
			</CardContent>
			<CardFooter className="flex gap-2">
				<SelectorFooter />
			</CardFooter>
		</Card>
	);
};
