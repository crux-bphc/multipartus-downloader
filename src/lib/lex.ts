export function fetchLex<T>([url, token]: [string, string]): Promise<T> {
	return fetch(`https://lex.local.crux-bphc.com/impartus/${url}`, {
		headers: {
			Authorization: `Bearer ${token}`,
		},
	}).then((res) => res.json());
}
