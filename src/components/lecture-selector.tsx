import { useEffect } from "react";
import { Controller, useFormContext } from "react-hook-form";
import useSWR from "swr";
import type { DownloadFormValues } from "./download-form";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "./ui/select";

const getStringValue = (lecture: Multipartus.Lecture) =>
	lecture.id.ID.join(";");

const getSession = (
	sessions: Multipartus.Sessions,
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
	const { control, setValue } = useFormContext<DownloadFormValues>();
	const { data: sessions } = useSWR<Multipartus.Sessions>("session");
	const { data: lectures } = useSWR<Multipartus.Lecture[]>(
		`subject/${props.department}/${props.code}/lectures`,
	);

	useEffect(() => {
		if (lectures) {
			setValue("lecture", lectures[0].id.ID);
		}
	}, [lectures]);

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
					defaultValue={field.value?.join(";")}
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
								{[
									lecture.section,
									lecture.professor,
									getSession(sessions, lecture),
								].join(" | ")}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			)}
		/>
	);
}
