<script context="module">
	/** @type {import('./__types/[slug]').Load} */
	export async function load({ url, params, fetch }) {
		const code = url.searchParams.get('code');
		const state = url.searchParams.get('state');

		const send_me = `http://127.0.0.1:4640/oauth/callback/${params.connection}?code=${code}&state=${state}&redirect_url=http://localhost:4639/oauth/callback/${params.connection}`;

		const response = await fetch(send_me);

		return {
			status: response.status,
			props: {
				response: await response.text()
			}
		};
	}
</script>

<script lang="ts">
	export let response: string;
</script>

<div class="container mx-auto mt-4">Connected: {response}</div>
