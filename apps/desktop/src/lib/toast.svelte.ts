// Svelte 5 global rune state for Toast Notification
export const toastState = $state({
    show: false,
    message: "",
    type: "success" as "success" | "error" | "warning" | "info"
});

let toastTimeout: ReturnType<typeof setTimeout> | null = null;

export function showToast(msg: string, type: "success" | "error" | "warning" | "info" = "success") {
    toastState.message = msg;
    toastState.type = type;
    toastState.show = true;
    
    if (toastTimeout) clearTimeout(toastTimeout);
    
    toastTimeout = setTimeout(() => {
        toastState.show = false;
    }, type === 'error' ? 4000 : 3000);
}
