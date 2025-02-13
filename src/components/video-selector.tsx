import { useLogto } from "@/lib/logto";
import { useEffect } from "react";
import { Controller, useFormContext } from "react-hook-form";
import useSWR from "swr";
import type { DownloadFormValues } from "./download-form";
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
	const { control, setValue } = useFormContext<DownloadFormValues>();

	useEffect(() => {
		if (videos) {
			setValue(
				"videos",
				videos.map((video) => ({ selected: true, ttid: video.ttid })),
			);
		}
	}, [videos]);

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
							<Controller
								name={`videos.${i}.selected`}
								control={control}
								render={({ field }) => (
									<Checkbox
										id={`video-${video.ttid}`}
										checked={field.value}
										onCheckedChange={field.onChange}
										ref={field.ref}
									/>
								)}
							/>
						</div>
					</div>
				</div>
			))}
		</div>
	);
};

export const VideoSelector = () => {
	const { getValues, setValue } = useFormContext<DownloadFormValues>();
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
				<Button
					type="button"
					variant="secondary"
					onClick={() => {
						setValue(
							"videos",
							getValues("videos").map((video) => ({
								...video,
								selected: false,
							})),
						);
					}}
				>
					Deselect All
				</Button>
				<Button
					type="button"
					onClick={() => {
						setValue(
							"videos",
							getValues("videos").map((video) => ({
								...video,
								selected: true,
							})),
						);
					}}
				>
					Select All
				</Button>
			</CardFooter>
		</Card>
	);
};
