<script lang="ts" context="module">
	import type { Load } from '@sveltejs/kit';

	export const load: Load = async ({ fetch }) => {
		const res = await fetch('/services');
		const data = await res.json();

		return { props: { ...data } };
	};
</script>

<script lang="ts">
	interface OAuthConnection {
		name: string;
		phase: string;
	}

	export let apis: string[];
	export let connections: OAuthConnection[];
</script>

<div class="container mx-auto mt-4">
	<h2>Avialable OAuth Connections</h2>
	{#each connections as connection}
		<div class="hover:bg-gray-200 cursor-pointer px-6 py-2 border-b border-gray-500">
			<h4 class="font-bold">{connection.name}</h4>
			<p class="text-gray-500">
				{connection.phase} -
				<a
					href="http://127.0.0.1:4640/oauth/connections/{connection.name}?redirect_url=http://localhost:4639/oauth/callback/{connection.name}"
					>Connect</a
				>
			</p>
		</div>
	{/each}
</div>
