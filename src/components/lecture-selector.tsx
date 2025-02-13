import useSWR from "swr";
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
	const { data: sessions } = useSWR<Multipartus.Sessions>("session");
	const { data: lectures } = useSWR<Multipartus.Lecture[]>(
		`subject/${props.department}/${props.code}/lectures`,
	);

	if (!lectures || !sessions) {
		return <div>Loading...</div>;
	}

	return (
		<Select>
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
	);
}
