import { invoke as rawInvoke } from "@tauri-apps/api";
import { type InvokeArgs } from "@tauri-apps/api/tauri";
import { message } from "@tauri-apps/api/dialog";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { database } from "../stores";

import { type ConnectionDirection } from "./ConnectionDirection";
import { type ConnectionInfo } from "./ConnectionInfo";
import { type DatabaseInfo } from "./DatabaseInfo";
import { type Device } from "./Device";
import { type IpRange } from "./IpRange";
import { type GenCoordinate } from "./GenCoordinate";
import { type LocationInfo } from "./LocationInfo";

type Coordinate = GenCoordinate<number>;
type ThreadID = string;

const errorDialog = (msg: string): Promise<void> =>
    message(`Error: ${msg}`, { title: "Error", type: "error" });

/** invoke a tauri command, showing the error on screen if error returned */
const invoke = async (cmd: string, args?: InvokeArgs | undefined): Promise<any> => {
    try {
        return await rawInvoke(cmd, args);
    } catch (e) {
        await errorDialog(e as string);

        throw e;
    }
};

/** Corresponding definitions in /backend/src/capture.rs */
const capture = {
    /** The list of all available network devices. */
    listDevices: (): Promise<Device[]> =>
        invoke("list_devices"),

    /** Start capturing on a certain network device */
    startCapturing: (device: Device): Promise<ThreadID> =>
        invoke("start_capturing", { name: device.name }),

    /** Stop capturing a certain capture thread */
    stopCapturing: (threadId: ThreadID): Promise<void> =>
        invoke("stop_capturing", { threadId }),

    /** A list of all connections over the length of the session */
    allConnections: (): Promise<ConnectionInfo[]> =>
        invoke("all_connections"),

    /** A list of connections currently happening in the session */
    currentConnections: (): Promise<ConnectionInfo[]> =>
        invoke("current_connections"),

    /** Runs a handler every time a new ip address connects */
    onNewConnection: (handler: (ip: string) => void): Promise<UnlistenFn> =>
        listen("new_connection", (event) => handler(event.payload as string))
};

/** Corresponding definitions in /backend/src/geoip.rs */
const geoip = {
    /** Load a database from a path on disk */
    loadDatabase: (path: string | string[] | null): Promise<DatabaseInfo | null> =>
        invoke("load_database", { path }),

    /** Delete a database from the global state, freeing up memory */
    unloadDatabase: (path: string): Promise<void> =>
        invoke("unload_database", { path }),

    /** List all databases (by info) */
    listDatabases: (): Promise<DatabaseInfo[]> =>
        invoke("list_databases"),

    /** Lookup the coordinate for an IP address in the database */
    lookupIp: (ip: string): Promise<Coordinate | null> =>
        invoke("lookup_ip", { ip, database: database.path() }),

    /** Find the range in the database for a given IP */
    lookupIpRange: (ip: string): Promise<IpRange | null> =>
        invoke("lookup_ip_range", { database: database.path(), ip }),

    /** Finds the block of ips for a given coordinate in the database */
    lookupIpBlocks: (coord: Coordinate): Promise<IpRange[]> =>
        invoke("lookup_ip_blocks", { coord, database: database.path() }),

    /** The nearest location in the database from a given coordinate */
    nearestLocation: (coord: Coordinate): Promise<Coordinate> =>
        invoke("nearest_location", { database: database.path(), coord }),

    /** Associated City, State, and Country for a Coordinate */
    locationInfo: (coord: Coordinate): Promise<LocationInfo | null> =>
        invoke("location_info", { database: database.path(), coord }),

    /** Our coordinate based on the current database */
    myLocation: (): Promise<Coordinate> =>
        invoke("my_location", { database: database.path() }),

    /** Lookup the associated DNS address with a string */
    lookupDns: (ip: string): Promise<string | null> =>
        invoke("dns_lookup_addr", { ip }),

    /** Validate if a string is a global IPv4 address */
    validateIp: (ip: string): Promise<boolean> =>
        invoke("validate_ip", { ip }),
};

export {
    type ConnectionDirection,
    type ConnectionInfo,
    type DatabaseInfo,
    type Device,
    type ThreadID,
    type IpRange,
    type Coordinate,
    type LocationInfo,

    errorDialog,

    capture,
    geoip,
};
