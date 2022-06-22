import type { EndpointOutput } from '@sveltejs/kit';

export async function get(): Promise<EndpointOutput> {
	const res = await fetch('http://127.0.0.1:7979/oauth/apis');
	const data = await res.json();

	const res2 = await fetch('http://127.0.0.1:7979/oauth/connections');
	const data2 = await res2.json();

	console.log(data2);

	return {
		body: {
			apis: data,
			connections: data2
		}
	};
}
