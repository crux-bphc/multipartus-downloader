import { fetchLex } from "@/lib/lex";
import { atom, useAtomValue } from "jotai";
import { loadable } from "jotai/utils";
import { lectureAtom } from "./download-form";
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
	atom(async (get) => {
		const lecture = get(lectureAtom);
		if (!lecture) {
			return [];
		}

		const videos = await fetchLex<Multipartus.Video[]>(
			`lecture/${lecture.join("/")}`,
		);

		return videos;
	}),
);

const Videos = () => {
	const videos = useAtomValue(videosAtom);

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
							<Checkbox id={`video-${video.ttid}`} />
						</div>
					</div>
				</div>
			))}
		</div>
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
				<span className="text-muted-foreground text-bold mr-auto">
					(0) Selected
				</span>
				<Button type="button" variant="secondary">
					Deselect All
				</Button>
				<Button type="button">Select All</Button>
			</CardFooter>
		</Card>
	);
};
