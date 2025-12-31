import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Item } from '../types';

const items = ref<Item[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

export function useItems() {
  const fetchItems = async (stateFilter?: string, groupFilter?: string) => {
    loading.value = true;
    error.value = null;
    try {
      items.value = await invoke<Item[]>('get_items', { stateFilter, groupFilter });
    } catch (e) {
      error.value = e as string;
      console.error('Failed to fetch items:', e);
    } finally {
      loading.value = false;
    }
  };

  const fetchItem = async (id: number): Promise<Item | null> => {
    try {
      return await invoke<Item>('get_item', { id });
    } catch (e) {
      error.value = e as string;
      console.error('Failed to fetch item:', e);
      return null;
    }
  };

  const updateItemState = async (id: number, state: 'unread' | 'read' | 'archived') => {
    try {
      await invoke('update_item_state', { id, state });
      // Update local state
      const item = items.value.find(i => i.id === id);
      if (item) {
        item.state = state;
      }
    } catch (e) {
      error.value = e as string;
      console.error('Failed to update item state:', e);
    }
  };

  const unreadCount = computed(() => {
    return items.value.filter(i => i.state === 'unread').length;
  });

  return {
    items,
    loading,
    error,
    fetchItems,
    fetchItem,
    updateItemState,
    unreadCount,
  };
}

