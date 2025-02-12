import { useLogto } from "@/lib/logto";
import useSWR from "swr";
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

const Videos = (props: { session: number; subjectId: number }) => {
	const { idToken } = useLogto();
	const { data: videos } = useSWR<Multipartus.Video[]>([
		`lecture/${props.session}/${props.subjectId}`,
		idToken,
	]);
	return (
		<div className="grid grid-cols-3 gap-2">
			{videos?.toReversed().map((video, i) => (
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
									{i + 1}
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
					<Videos session={1249} subjectId={2628604} />
				</ScrollArea>
			</CardContent>
			<CardFooter className="flex gap-2">
				<Button disabled variant="link" className="mr-auto">
					(0) Selected
				</Button>
				<Button variant="secondary">Deselect All</Button>
				<Button>Select All</Button>
			</CardFooter>
		</Card>
	);
};
