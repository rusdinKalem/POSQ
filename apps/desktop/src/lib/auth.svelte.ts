import { invoke } from "@tauri-apps/api/core";

export interface UserSession {
    session_id: string;
    session_token: string;
    user_id: string;
    user_name: string;
    outlet_id: string | null;
    roles: string[];
}

export const authState = $state<{
    session: UserSession | null;
    loading: boolean;
}>({
    session: null,
    loading: true
});

export async function refreshSession() {
    authState.loading = true;
    try {
        const session = await invoke<UserSession | null>("get_active_session");
        authState.session = session;
    } catch (e) {
        console.error("Failed to get active session:", e);
        authState.session = null;
    } finally {
        authState.loading = false;
    }
}

export async function login(pin: string, deviceId = "default_device", registerId = "default_register") {
    try {
        const session = await invoke<UserSession>("login_user", {
            pin,
            deviceId,
            registerId
        });
        authState.session = session;
        return session;
    } catch (e) {
        console.error("Login failed:", e);
        throw e;
    }
}

export async function logout() {
    try {
        if (authState.session) {
            // Sesuai dengan backend, logout_user memerlukan `session_token`
            // Namun, get_active_session mengembalikan `session_token` kosong ("").
            // Untuk memastikan logout berhasil di backend tanpa error, kita kirim token jika ada.
            await invoke("logout_user", { sessionToken: authState.session.session_token || "" });
        }
    } catch (e) {
        console.error("Logout backend failed:", e);
    } finally {
        authState.session = null;
    }
}
