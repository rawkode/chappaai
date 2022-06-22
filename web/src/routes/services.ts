import type { EndpointOutput } from '@sveltejs/kit';

export async function get(): Promise<EndpointOutput> {
	const res = await fetch('http://127.0.0.1:4640/oauth/connections');
	const data = await res.json();

	return {
		body: {
			connections: data
		}
	};
}
