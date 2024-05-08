<script>
  import Settings from "./Settings.svelte";
  import Toasts from "./Toasts.svelte";
  import IpTable from "./Map.svelte";
</script>

<main class="p-4 m-0 space-y-4 w-screen h-screen flex flex-col">
  <h1 class="text-3xl font-bold italic">ipmap :)</h1>
  <Settings />
  <IpTable />
  <Toasts />
</main>
