<script>
    import { writable } from "svelte/store";
    import { listen } from "@tauri-apps/api/event";
    import { fade } from "svelte/transition";

    let toasts = (() => {
        const { subscribe, update } = writable([]);

        const randomID = () => (Math.random() + 1).toString(36).substring(2);
        const add = (toast) =>
            update((toasts) => (toasts = [...toasts, toast]));

        return {
            subscribe,
            newError: (msg) => add({ msg, id: randomID(), error: true }),
            newInfo: (msg) => add({ msg, id: randomID(), error: false }),
            remove: (toast) =>
                update((toasts) => (toasts = toasts.filter((t) => t != toast))),
        };
    })();

    listen("error", (e) => toasts.newError(e.payload));
    listen("info", (e) => toasts.newInfo(e.payload));
</script>

<div class="toast toast-end z-[9999]">
    {#each $toasts as toast}
        <div
            role="alert"
            class="alert py-2 flex"
            class:alert-error={toast.error}
            class:alert-info={!toast.error}
            out:fade={{ duration: 100 }}
        >
            <span class="grow font-semibold">{!toast.error ? "Info:" : ""} {toast.msg}</span>
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
