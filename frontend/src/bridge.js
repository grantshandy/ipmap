import { invoke as rawInvoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";

// invoke a tauri command, showing the error on screen if error returned
export const invoke = async (cmd, args) => {
    try {
        return await rawInvoke(cmd, args);
    } catch (e) {
        emit("error", e);
        throw e;
    }
};

export const setDatabase = async (path) => invoke("set_database", { path });
export const setDevice = async (name) => invoke("set_device", { name });
export const listDevices = async () => invoke("list_devices");

export const stopCapturing = async () => invoke("stop_capturing");
export const startCapturing = async () => invoke("start_capturing");

export const lookupIp = async (ip) => invoke("lookup_ip", { ip });