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
}
