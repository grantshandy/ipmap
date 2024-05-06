import { writable } from "svelte/store";

export const createToasts = () => {
    const { subscribe, update } = writable([]);

    const randomID = () => (Math.random() + 1).toString(36).substring(2);
    const add = (toast) => update(toasts => toasts = [...toasts, toast]);

    return {
        subscribe,
        newError: (msg) => add({ msg, id: randomID(), error: true }),
        newInfo: (msg) => add({ msg, id: randomID(), error: false }),
        remove: (toast) => update(toasts => toasts = toasts.filter((t) => t != toast))
    };
};