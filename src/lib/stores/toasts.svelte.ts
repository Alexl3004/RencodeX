export type ToastType = "success" | "error" | "warn" | "info";

export interface ToastAction {
  label: string;
  onClick: () => void;
}

export interface Toast {
  id:       number;
  type:     ToastType;
  title?:   string;
  message:  string;
  duration: number;      // ms, 0 = persistant
  action?:  ToastAction;
  count:    number;      // nb de fois que ce message a été empilé
  addedAt:  number;      // timestamp pour la barre de progression
}

let _id = 0;

function createToasts() {
  let items = $state<Toast[]>([]);

  function add(
    type: ToastType,
    message: string,
    opts: { title?: string; duration?: number; action?: ToastAction } = {},
  ): number {
    const duration = opts.duration ?? (type === "error" ? 6000 : type === "warn" ? 5000 : 4000);

    // Dédoublonnage : si le même message existe déjà, on incrémente son compteur
    // et on réinitialise son timer plutôt que d'empiler un doublon.
    const existing = items.find(t => t.type === type && t.message === message);
    if (existing) {
      existing.count++;
      existing.addedAt = Date.now();
      items = [...items];
      return existing.id;
    }

    const id = ++_id;
    const toast: Toast = {
      id,
      type,
      title:   opts.title,
      message,
      duration,
      action:  opts.action,
      count:   1,
      addedAt: Date.now(),
    };

    // Max 5 toasts visibles — retire le plus ancien si dépassé
    if (items.length >= 5) {
      items = [...items.slice(1), toast];
    } else {
      items = [...items, toast];
    }

    if (duration > 0) {
      setTimeout(() => remove(id), duration);
    }

    return id;
  }

  function remove(id: number) {
    items = items.filter(t => t.id !== id);
  }

  function clear() {
    items = [];
  }

  return {
    get items() { return items; },

    success(message: string, opts?: { title?: string; duration?: number; action?: ToastAction }) {
      return add("success", message, opts);
    },
    error(message: string, opts?: { title?: string; duration?: number; action?: ToastAction }) {
      return add("error", message, { duration: 0, ...opts });
    },
    warn(message: string, opts?: { title?: string; duration?: number; action?: ToastAction }) {
      return add("warn", message, opts);
    },
    info(message: string, opts?: { title?: string; duration?: number; action?: ToastAction }) {
      return add("info", message, opts);
    },
    remove,
    clear,
  };
}

export const toasts = createToasts();