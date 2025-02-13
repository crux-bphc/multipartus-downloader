import { fetchLex } from "@/lib/lex";
import { atom, useAtom, useAtomValue } from "jotai";
import { loadable } from "jotai/utils";
import { useEffect } from "react";
import { lectureAtom, subjectAtom } from "./download-form";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "./ui/select";

const getSessionLabel = (
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

const sessionsAtom = atom(async () => {
	const sessions = await fetchLex<Multipartus.Sessions>("session");
	return sessions;
});

const lecturesAtom = loadable(
	atom(async (get) => {
		const subject = get(subjectAtom);
		const sessions = await get(sessionsAtom);
		const lectures = await fetchLex<Multipartus.Lecture[]>(
			`subject/${subject?.join("/")}/lectures`,
		);

		return lectures.map((lecture) => ({
			id: lecture.id.ID,
			value: lecture.id.ID.join(";"),
			label: [
				lecture.section,
				lecture.professor,
				getSessionLabel(sessions, lecture),
			].join(" | "),
		}));
	}),
);

export function LectureSelector() {
	const [lecture, setLecture] = useAtom(lectureAtom);
	const lectures = useAtomValue(lecturesAtom);

	useEffect(() => {
		if (lectures.state === "hasData" && lectures.data.length > 0) {
			setLecture(lectures.data[0].id);
		}
	}, [lectures]);

	if (lectures.state !== "hasData") {
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
				{lectures.data.map((lecture) => (
					<SelectItem key={lecture.value} value={lecture.value}>
						{lecture.label}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	);
}
