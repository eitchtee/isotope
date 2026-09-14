import type { DragItem } from "./dnd";

/** The item currently being dragged, shared by the sidebar and the folder panel. */
export const dragging = $state<{ current: DragItem | null }>({ current: null });
