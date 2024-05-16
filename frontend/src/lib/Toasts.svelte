<script lang="ts">
    import { writable } from "svelte/store";
    import type { Writable } from "svelte/store";
    import { listen } from "@tauri-apps/api/event";
    import { fade } from "svelte/transition";
    import CloseIcon from "./CloseIcon.svelte";

    type Toast = {
        msg: string,
        id: string,
        error: boolean,
    };

    type ToastStore = {
        subscribe: Writable<Toast[]>["subscribe"],
        newError: (msg: string) => void,
        newInfo: (msg: string) => void,
        remove: (toast: Toast) => void,
    };

    let toasts = ((): ToastStore => {
        const { subscribe, update } = writable<Toast[]>([]);

        const randomID = () => (Math.random() + 1).toString(36).substring(2);
        const add = (toast: Toast): void =>
            update((toasts: Toast[]) => (toasts = [...toasts, toast]));

        return {
            subscribe,
            newError: (msg) => add({ msg, id: randomID(), error: true }),
            newInfo: (msg) => add({ msg, id: randomID(), error: false }),
            remove: (toast) =>
                update((toasts) => (toasts = toasts.filter((t) => t != toast))),
        };
    })();

    listen("error", (e) => toasts.newError(e.payload as string));
    listen("info", (e) => toasts.newInfo(e.payload as string));
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
                <CloseIcon />
            </button>
        </div>
    {/each}
</div>
