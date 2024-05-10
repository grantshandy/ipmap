import { invoke as rawInvoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";
import type { InvokeArgs } from "@tauri-apps/api/tauri";

type Device = {
    name: string,
    desc: string | null,
    prefered: boolean
};

export type LocationSelection = {
    loc: Location,
    ips: string[],
    marker: any
}

export type Location = {
    latitude: number,
    longitude: number,
    city: string | null,
    country_code: string | null,
    timezone: string | null,
    state: string | null
}

// invoke a tauri command, showing the error on screen if error returned
export const invoke = async (cmd: string, args?: InvokeArgs | undefined): Promise<any> => {
    try {
        return await rawInvoke(cmd, args);
    } catch (e) {
        emit("error", e);
        throw e;
    }
};

export const listDevices = async (): Promise<Device[]> => invoke("list_devices");
export const setDatabase = async (path?: string): Promise<string | null> => invoke("set_database", { path });
export const setDevice = async (name?: string): Promise<void> => invoke("set_device", { name });

export const stopCapturing = async (): Promise<void> => invoke("stop_capturing");
export const startCapturing = async (): Promise<void> => invoke("start_capturing");

export const lookupIp = async (ip: string): Promise<Location | null> => invoke("lookup_ip", { ip });