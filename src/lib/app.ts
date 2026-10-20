import { invoke } from "@tauri-apps/api/core";

export const setHotkey = (accelerator: string) => invoke<void>("set_hotkey", { accelerator });
export const getScreenPermission = () => invoke<boolean>("get_screen_permission");
export const requestScreenPermission = () => invoke<boolean>("request_screen_permission");
