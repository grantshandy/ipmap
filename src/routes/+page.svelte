<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
    import { openUrl } from "@tauri-apps/plugin-opener";

  let name = $state("");
</script>

<main>
  <h1 class="text-2xl font-bold">
    Welcome to Tauri + Svelte + Tailwindcss + Daisyui
  </h1>

  <form
    class="join join-horizontal"
    onsubmit={async (event: Event) => {
      event.preventDefault();
      alert(await invoke("greet", { name }));
    }}
  >
    <input
      class="input input-sm join-item"
      placeholder="Enter a name..."
      bind:value={name}
    />
    <button type="submit" class="btn btn-sm btn-primary join-item">Greet</button
    >
  </form>


  <img src="/svelte.svg" alt="logo" />

  <button class="link" onclick={() => openUrl("https://google.com")}>visit</button>

</main>
