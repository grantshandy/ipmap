<script lang="ts">
  import { invoke } from "@tauri-apps/api";

  let number = 0;

  $: invoke("print", { name: number.toString() });
</script>

<main>
  <h1>Ipmap</h1>
  <button on:click={() => (number += 1)}>+</button>
  <button on:click={() => (number -= 1)}>-</button>
  <p>number: {number}</p>
</main>
