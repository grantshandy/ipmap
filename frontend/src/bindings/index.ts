import { invoke as rawInvoke } from "@tauri-apps/api";
import type { InvokeArgs } from "@tauri-apps/api/tauri";
import { message } from "@tauri-apps/api/dialog";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { type ConnectionDirection } from "./ConnectionDirection";
import { type ConnectionInfo } from "./ConnectionInfo";
import { type DatabaseInfo } from "./DatabaseInfo";
import { type Device } from "./Device";
import { type Location } from "./Location";
import { type IpRange } from "./IpRange";

const errorDialog = (msg: string): Promise<void> => {
    return message(`Error: ${msg}`, { title: "Error", type: "error" });
};

// invoke a tauri command, showing the error on screen if error returned
const invoke = async (cmd: string, args?: InvokeArgs | undefined): Promise<any> => {
    try {
        return await rawInvoke(cmd, args);
    } catch (e) {
        await errorDialog(e as string);

        throw e;
    }
};

const listDevices = async (): Promise<Device[]> => invoke("list_devices");

type ThreadID = string;

const stopCapturing = async (threadId: ThreadID): Promise<void> => invoke("stop_capturing", { threadId });
const startCapturing = async (name: string): Promise<ThreadID> => invoke("start_capturing", { name });
const currentConnections = async (): Promise<ConnectionInfo[]> => invoke("current_connections");
const allConnections = async (): Promise<ConnectionInfo[]> => invoke("all_connections");

const loadDatabase = async (path: string | string[] | null): Promise<DatabaseInfo | null> => invoke("load_database", { path });
const unloadDatabase = async (path: string) => invoke("unload_database", { path });
const listDatabases = async (): Promise<DatabaseInfo[]> => invoke("list_databases");
const lookupIp = async (ip: string, database: DatabaseInfo | null): Promise<Location | null> => invoke("lookup_ip", { ip, database: database?.path });
const lookupIpRange = async (ip: string, database: DatabaseInfo | null): Promise<IpRange | null> => invoke("lookup_ip_range", { database: database?.path, ip });
const nearestLocation = async (latitude: number, longitude: number, database: DatabaseInfo | null): Promise<Location | null> => invoke("nearest_location", { database: database?.path, latitude, longitude })

const lookupDns = async (ip: string): Promise<string | null> => invoke("dns_lookup_addr", { ip });
const validateIp = async (ip: string): Promise<boolean> => invoke("validate_ip", { ip });
const myLocation = async (database: DatabaseInfo | null): Promise<Location> => invoke("my_location", { database: database?.path });

const onNewConnection = (handler: (ip: string) => void): Promise<UnlistenFn> =>
    listen("new_connection", (event) => handler(event.payload as string));

export {
    type ConnectionDirection,
    type ConnectionInfo,
    type DatabaseInfo,
    type Device,
    type Location,
    type ThreadID,
    type IpRange,

    errorDialog,

    listDevices,

    stopCapturing,
    startCapturing,
    currentConnections,
    allConnections,
    onNewConnection,

    loadDatabase,
    unloadDatabase,
    listDatabases,
    lookupIpRange,
    nearestLocation,

    lookupIp,
    lookupDns,
    validateIp,
    myLocation,
};
