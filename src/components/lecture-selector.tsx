import { lectureAtom, subjectAtom } from "@/lib/atoms";
import { fetchLex } from "@/lib/lex";
import { atom, useAtom, useAtomValue } from "jotai";
import { loadable } from "jotai/utils";
import { useEffect } from "react";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "./ui/select";
import { Skeleton } from "./ui/skeleton";

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

const sessionsAtom = atom(async (_, { signal }) => {
	const sessions = await fetchLex<Multipartus.Sessions>("session", { signal });
	return sessions;
});

const lecturesAtom = loadable(
	atom(async (get) => {
		const subject = get(subjectAtom);

		if (!subject) {
			return [];
		}

		const sessions = await get(sessionsAtom);
		const lectures = await fetchLex<Multipartus.Lecture[]>(
			`subject/${subject[0].replaceAll("/", ",")}/${subject[1]}/lectures`,
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
	const [selectedLecture, selectLecture] = useAtom(lectureAtom);
	const lectures = useAtomValue(lecturesAtom);

	useEffect(() => {
		if (lectures.state === "hasData" && lectures.data.length > 0) {
			selectLecture(lectures.data[0].id);
		}
	}, [lectures]);

	if (lectures.state !== "hasData") {
		return <Skeleton className="h-9 flex-grow" />;
	}

	return (
		<Select
			onValueChange={(value) =>
				selectLecture(value.split(";").map(Number) as [number, number])
			}
			value={selectedLecture ? selectedLecture.join(";") : undefined}
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
