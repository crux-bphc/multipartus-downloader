import useSWR from "swr";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "./ui/select";
import { useLogto } from "@/lib/logto";

export function LectureSelector(props: { department: string; code: string }) {
	const { idToken } = useLogto();
	const { data: lectures } = useSWR<Multipartus.Lecture[]>([
		`subject/${props.department}/${props.code}/lectures`,
		idToken,
	]);
	const { data: sessions } = useSWR<Record<string, [number, number]>>([
		"session",
		idToken,
	]);

	if (!lectures || !sessions) {
		return <div>Loading...</div>;
	}

	const getStringValue = (lecture: Multipartus.Lecture) =>
		lecture.id.ID.join(";");

	const getSession = (lecture: Multipartus.Lecture) => {
		const session = lecture.impartus_session.toString();
		if (session in sessions) {
			const [year, sem] = sessions[session];
			return `${year} - ${year + 1} | Sem ${sem}`;
		}
		return "unknown session";
	};

	return (
		<Select defaultValue={getStringValue(lectures[0])}>
			<SelectTrigger>
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				{lectures.map((lecture) => (
					<SelectItem
						key={getStringValue(lecture)}
						value={getStringValue(lecture)}
					>
						{lecture.section} | {lecture.professor} | {getSession(lecture)}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	);
}
