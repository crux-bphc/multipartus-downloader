import { logtoClient } from "./logto";

export async function fetchLex<T>(url: string): Promise<T> {
	return fetch(`https://lex.crux-bphc.com/api/impartus/${url}`, {
		headers: {
			Authorization: `Bearer ${await logtoClient.getIdToken()}`,
		},
	}).then((res) => res.json());
}
