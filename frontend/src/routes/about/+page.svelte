<script lang="ts">
  import "../../tailwind.css";
  import { open } from "@tauri-apps/api/shell";
</script>

<div class="w-screen h-screen text-center flex flex-col space-y-2">
  <h1 class="text-xl font-semibold">About IPmap</h1>
  <p>Ipmap is free and open source software.</p>
  <div class="grow space-y-2">
    <p><button on:click={() => open("https://www.gnu.org/licenses/gpl-3.0.en.html")}>View License</button></p>
    <p><button on:click={() => open("https://github.com/grantshandy/ipmap")}>View Source Code</button></p>
  </div>
  <p class="text-sm italic">&copy; 2023 Grant Handy</p>
</div>

<style>
  button {
    @apply px-1 py-0.5 rounded-md bg-gray-300 border border-gray-500;
  }
</style>
