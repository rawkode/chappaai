import type { PageServerLoad } from "./$types";

interface Response {
	resposneCode: number;
	responseText: string;
}

export const load: PageServerLoad = async ({
	fetch,
	params,
	url,
}): Promise<Response> => {
	const code = url.searchParams.get("code");
	const state = url.searchParams.get("state");

	const send_me = `http://127.0.0.1:4640/oauth/callback/${params.connection}?code=${code}&state=${state}&redirect_url=http://localhost:4639/oauth/callback/${params.connection}`;

	const response = await fetch(send_me);

	return {
		resposneCode: response.status,
		responseText: await response.text(),
	};
};
