<script>
  import { listen, emit } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api";
  import { createToasts } from "./toasts";

  let toasts = createToasts();
  listen("error", (e) => toasts.newError(e.payload));

  // invoke a tauri command, showing the error on screen
  const invokeWrapper = async (cmd, args) =>
    await invoke(cmd, args).catch((e) => toasts.newError(e));

  let devices = null;
  invokeWrapper("device_list").then((x) => (devices = x));

  listen("new_connection", console.log);
</script>

<main class="p-4">
  {#if devices != null}
    <select
      class="select select-bordered select-sm w-full max-w-xs"
      on:change={(event) => emit("change_device", { name: event.target.value })}
    >
      <option disabled selected>Select Network Capture Device</option>
      {#each devices as device}
        <option value={device.name}>
          {#if device.desc != null}
            {device.desc}
          {:else}
            No Description ({device.name})
          {/if}
          {#if device.prefered}(Default){/if}
        </option>
      {/each}
    </select>
  {/if}

  <!-- Messages -->
  <div class="toast toast-end">
    {#each $toasts as toast}
      <div
        role="alert"
        class="alert flex"
        class:alert-error={toast.error}
        class:alert-info={!toast.error}
      >
        <span class="grow">{toast.error ? "Error" : "Info"}: {toast.msg}</span>
        <button
          class="btn btn-circle btn-xs btn-outline"
          on:click={() => toasts.remove(toast)}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-4 w-4"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            ><path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            /></svg
          >
        </button>
      </div>
    {/each}
  </div>
</main>
