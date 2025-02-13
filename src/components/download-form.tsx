import { fetchLex } from "@/lib/lex";
import { useLogto } from "@/lib/logto";
import { DownloadIcon } from "lucide-react";
import { FormProvider, type SubmitHandler, useForm } from "react-hook-form";
import { LectureSelector } from "./lecture-selector";
import { Button } from "./ui/button";
import { VideoSelector } from "./video-selector";

export type DownloadFormValues = {
	lecture: [number, number];
	videos: {
		selected: boolean;
		ttid: number;
	}[];
};

export function DownloadForm(props: { department: string; code: string }) {
	const { idToken } = useLogto();
	const methods = useForm<DownloadFormValues>({
		defaultValues: async () => {
			const lectures = await fetchLex<Multipartus.Lecture[]>([
				`subject/${props.department}/${props.code}/lectures`,
				// biome-ignore lint/style/noNonNullAssertion: has to be non-null
				idToken!,
			]);

			return {
				lecture: lectures[0].id.ID,
				videos: [],
			};
		},
	});

	const onSubmit: SubmitHandler<DownloadFormValues> = (data) => {
		console.log(data);
	};

	return (
		<FormProvider {...methods}>
			<form onSubmit={methods.handleSubmit(onSubmit)}>
				<div className="flex flex-col gap-6">
					<LectureSelector department={props.department} code={props.code} />
					<VideoSelector />
					<Button size="lg">
						Download
						<DownloadIcon />
					</Button>
				</div>
			</form>
		</FormProvider>
	);
}
