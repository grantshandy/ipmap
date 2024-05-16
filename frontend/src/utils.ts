import { invoke as rawInvoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";
import type { InvokeArgs } from "@tauri-apps/api/tauri";
import type { Marker } from "leaflet";

type Device = {
    name: string,
    desc: string | null,
    prefered: boolean
}

export type DatabaseInfo = {
    name: string,
    path: string | null,
    build_time: string,
    attribution_text: string | null,
    locations: number
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

export type Connection = {
    capturing_uuid: string,
    ip: string
};

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

export const stopCapturing = async (stopSignal: string): Promise<void> => invoke("stop_capturing", { stopSignal });
export const startCapturing = async (name: string): Promise<string> => invoke("start_capturing", { name });

export const loadDatabase = async (path: string | string[] | null): Promise<DatabaseInfo | null> => invoke("load_database", { path });
export const listDatabases = async (): Promise<DatabaseInfo[]> => invoke("list_databases");
export const lookupIp = async (ip: string, databasePath: string | null): Promise<Location | null> => invoke("lookup_ip", { ip, database: databasePath });
export const lookupDns = async (ip: string): Promise<string | null> => invoke("dns_lookup_addr", { ip });
