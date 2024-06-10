import { invoke as rawInvoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";
import type { InvokeArgs } from "@tauri-apps/api/tauri";

import { type Connection } from "./Connection";
import { type DatabaseInfo } from "./DatabaseInfo";
import { type Device } from "./Device";
import { type Location } from "./Location";

// invoke a tauri command, showing the error on screen if error returned
const invoke = async (cmd: string, args?: InvokeArgs | undefined): Promise<any> => {
    try {
        return await rawInvoke(cmd, args);
    } catch (e) {
        emit("error", e);
        throw e;
    }
};

const listDevices = async (): Promise<Device[]> => invoke("list_devices");

const stopCapturing = async (stopSignal: string): Promise<void> => invoke("stop_capturing", { stopSignal });
const startCapturing = async (name: string): Promise<string> => invoke("start_capturing", { name });

const loadDatabase = async (path: string | string[] | null): Promise<DatabaseInfo | null> => invoke("load_database", { path });
const listDatabases = async (): Promise<DatabaseInfo[]> => invoke("list_databases");
const lookupIp = async (ip: string, database: DatabaseInfo | null): Promise<Location | null> => invoke("lookup_ip", { ip, database: database?.path });

const lookupDns = async (ip: string): Promise<string | null> => invoke("dns_lookup_addr", { ip });
const validateIp = async (ip: string): Promise<boolean> => invoke("validate_ip", { ip });

export {
    type Connection,
    type DatabaseInfo,
    type Device,
    type Location,

    listDevices,

    stopCapturing,
    startCapturing,

    loadDatabase,
    listDatabases,
    lookupIp,

    lookupDns,
    validateIp
};