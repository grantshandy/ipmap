<script lang="ts">
  import CaptureDevices from "$lib/CaptureDevices.svelte";
  import Databases from "$lib/Databases.svelte";
  import CaptureMap from "$lib/CaptureMap.svelte";
    import ActiveList from "$lib/ActiveList.svelte";
</script>

<main class="min-h-screen p-3 space-y-3">
  <Databases />

  <hr />

  <CaptureDevices />
  <CaptureMap />
  <!-- <ActiveList /> -->
</main>
