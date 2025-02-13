import { useLogto } from "@/lib/logto";
import useSWR from "swr";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "./ui/select";
import { Controller, useFormContext } from "react-hook-form";
import type { DownloadFormValues } from "./download-form";
import { useEffect } from "react";

const getStringValue = (lecture: Multipartus.Lecture) =>
	lecture.id.ID.join(";");

const getSession = (
	sessions: Record<string, [number, number]>,
	lecture: Multipartus.Lecture,
) => {
	const session = lecture.impartus_session.toString();
	if (session in sessions) {
		const [year, sem] = sessions[session];
		return `${year} - ${year + 1} | Sem ${sem}`;
	}
	return "unknown session";
};

export function LectureSelector(props: { department: string; code: string }) {
	const { idToken } = useLogto();
	const { data: sessions } = useSWR<Record<string, [number, number]>>([
		"session",
		idToken,
	]);
	const { data: lectures } = useSWR<Multipartus.Lecture[]>([
		`subject/${props.department}/${props.code}/lectures`,
		idToken,
	]);

	const { control, setValue } = useFormContext<DownloadFormValues>();
	useEffect(() => {
		if (lectures) {
			setValue("lecture", lectures[0].id.ID);
		}
	}, [lectures])

	if (!lectures || !sessions) {
		return <div>Loading...</div>;
	}

	return (
		<Controller
			control={control}
			name="lecture"
			render={({ field }) => (
				<Select
					onValueChange={(value) => field.onChange(value.split(";"))}
					defaultValue={getStringValue(lectures[0])}
				>
					<SelectTrigger>
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{lectures.map((lecture) => (
							<SelectItem
								key={getStringValue(lecture)}
								value={getStringValue(lecture)}
							>
								{lecture.section} | {lecture.professor} |{" "}
								{getSession(sessions, lecture)}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			)}
		/>
	);
}
