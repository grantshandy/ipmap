import { commands, PLATFORM as PLATFORM_CONST, type Platform } from "./raw";

export const openAboutWindow = commands.openAboutWindow;
export const PLATFORM: Platform = PLATFORM_CONST as Platform;

export type * from "./raw";
export { APP_VERSION } from "./raw";
