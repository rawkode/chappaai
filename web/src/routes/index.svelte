<script lang="ts" context="module">
	import type { Load } from '@sveltejs/kit';

	export const load: Load = async ({ fetch }) => {
		const res = await fetch('/services');
		const data = await res.json();

		return { props: { ...data } };
	};
</script>

<script lang="ts">
	export let apis: string[];
	export let connections: string[];
</script>

<div class="container mx-auto mt-4">
	<h2>Available APIs</h2>
	{#each apis as api}
		<div class="hover:bg-gray-200 cursor-pointer px-6 py-2 border-b border-gray-500">
			<h4 class="font-bold">{api}</h4>
		</div>
	{/each}

	<h2>Connections</h2>
	{#each connections as connection}
		<div class="hover:bg-gray-200 cursor-pointer px-6 py-2 border-b border-gray-500">
			<h4 class="font-bold">{connection}</h4>
			<p class="text-gray-500">Disconnected</p>
		</div>
	{/each}
</div>
