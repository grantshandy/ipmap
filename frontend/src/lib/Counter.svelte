<script lang="ts">
  import { invoke } from "@tauri-apps/api";

  let count = 0;

  const increment = () => {
    count++;
    invoke('increment');
  };

</script>

<button on:click={increment}>
  count is {count}
</button>
