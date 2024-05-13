import { invoke as rawInvoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";
import type { InvokeArgs } from "@tauri-apps/api/tauri";
import type { Marker } from "leaflet";

type Device = {
    name: string,
    desc: string | null,
    prefered: boolean
}

type DatabaseInfo = {
    filename: string,
    built: string,
    attribution: string
}

export type LocationSelection = {
    loc: Location,
    ips: string[],
    marker: Marker,
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

export const stopCapturing = async (name: string): Promise<void> => invoke("stop_capturing", { name });
export const startCapturing = async (name: string): Promise<void> => invoke("start_capturing", { name });

export const loadDatabase = async (): Promise<void> => invoke("load_database");
export const builtinDatabaseInfo = async (): Promise<DatabaseInfo | null> => invoke("builtin_db_info");
export const lookupIp = async (ip: string): Promise<Location | null> => invoke("lookup_ip", { ip });
export const lookupDns = async (ip: string): Promise<string | null> => invoke("dns_lookup_addr", { ip });
