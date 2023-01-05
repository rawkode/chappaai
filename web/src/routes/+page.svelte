<script>
  /** @type {import('./$types').PageData} */
  export let data;
</script>

<div class="container mx-auto mt-4">
  <h2>Avialable OAuth Connections</h2>
  {#if data.connections.length === 0}
    <p class="text-gray-500">No connections found</p>
  {:else}
    {#each data.connections as connection}
      <div
        class="hover:bg-gray-200 cursor-pointer px-6 py-2 border-b border-gray-500"
      >
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
  {/if}
</div>
