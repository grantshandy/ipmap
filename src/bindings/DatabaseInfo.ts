// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { DatabaseType } from "./DatabaseType";
import type { IpType } from "./IpType";

/**
 * The associated metadata for a given Database
 */
export type DatabaseInfo = {
  name: string;
  kind: IpType;
  query: DatabaseType;
  attribution_text: string | null;
  build_time: string;
  unique_locations: number;
  strings: number;
};
