export const fetchLex = ([url, token]: [string, string]) => {
	return fetch(`https://lex.local.crux-bphc.com/impartus/${url}`, {
		headers: {
			Authorization: `Bearer ${token}`,
		},
	}).then((res) => res.json());
};
