namespace Multipartus {
	export interface Subject {
		id: {
			Table: string;
			ID: [string, string];
		};
		department: string;
		code: string;
		name: string;
	}

	export interface Lecture {
		id: {
			Table: string;
			ID: [number, number];
		};
		impartus_session: number;
		impartus_subject: number;
		section: string;
		professor: string;
	}

	export interface Video {
		ttid: number;
		topic: string;
		startTime: string;
		selected: boolean;
		number: number;
		subjectID: [string, string];
	}

	export type Sessions = Record<string, [number, number]>;
}
