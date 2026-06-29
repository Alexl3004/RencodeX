export type ToastType = "success" | "error" | "warn" | "info";

export interface ToastItem {
  id: number;
  message: string;
  type: ToastType;
}

const TOAST_CONFIG: Record<ToastType, { duration: number }> = {
  success: { duration: 4000 },
  error:   { duration: 6000 },
  warn:    { duration: 5000 },
  info:    { duration: 3500 },
};

function createToasts() {
  let items   = $state<ToastItem[]>([]);
  let counter = 0;

  function push(message: string, type: ToastType) {
    const id       = ++counter;
    const duration = TOAST_CONFIG[type].duration;
    items = [...items, { id, message, type }];
    setTimeout(() => {
      items = items.filter(t => t.id !== id);
    }, duration);
  }

  return {
    get items() { return items; },
    remove:  (id: number)  => { items = items.filter(t => t.id !== id); },
    success: (msg: string) => push(msg, "success"),
    error:   (msg: string) => push(msg, "error"),
    warn:    (msg: string) => push(msg, "warn"),
    info:    (msg: string) => push(msg, "info"),
  };
}

export const toasts = createToasts();