import type { PageServerLoad } from "./$types";

interface OAuthConnection {
	name: string;
	phase: string;
}

interface Response {
	apis: string[];
	connections: OAuthConnection[];
}

export const load: PageServerLoad = async ({ fetch }): Promise<Response> => {
	const res = await fetch("http://127.0.0.1:4640/oauth/connections", {
		headers: {
			origin: "http://localhost:5173",
		},
	});
	const data = await res.json();

	return {
		apis: [],
		connections: data,
	};
};
