import { useAtom } from "jotai";
import { useEffect } from "react";
import useSWR from "swr";
import { lectureAtom } from "./download-form";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "./ui/select";

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
	const [lecture, setLecture] = useAtom(lectureAtom);
	const { data: sessions } = useSWR<Multipartus.Sessions>("session");
	const { data: lectures } = useSWR<Multipartus.Lecture[]>(
		`subject/${props.department}/${props.code}/lectures`,
	);

	useEffect(() => {
		if (lectures && lectures.length > 0) {
			setLecture(lectures[0].id.ID);
		}
	}, [lectures]);

	if (!lectures || !sessions) {
		return <div>Loading...</div>;
	}

	return (
		<Select
			onValueChange={(value) =>
				setLecture(value.split(";").map(Number) as [number, number])
			}
			value={lecture ? lecture.join(";") : undefined}
		>
			<SelectTrigger>
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				{lectures.map((lecture) => (
					<SelectItem
						key={lecture.id.ID.join(";")}
						value={lecture.id.ID.join(";")}
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
	);
}
