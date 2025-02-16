import { logtoClient } from "./logto";

export async function fetchLex<T>(url: string, init?: RequestInit): Promise<T> {
	await logtoClient.getAccessToken();
	return fetch(`https://lex.crux-bphc.com/api/impartus/${url}`, {
		...init,
		headers: {
			Authorization: `Bearer ${await logtoClient.getIdToken()}`,
			...init?.headers,
		},
	}).then((res) => res.json());
}
