import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Item } from '../types';

const items = ref<Item[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

export function useItems() {
  const fetchItems = async (stateFilter?: string, groupFilter?: string, sourceIds?: number[], groupNames?: string[]) => {
    loading.value = true;
    error.value = null;
    try {
      items.value = await invoke<Item[]>('get_items', { 
        stateFilter, 
        groupFilter,
        sourceIds: sourceIds && sourceIds.length > 0 ? sourceIds : undefined,
        groupNames: groupNames && groupNames.length > 0 ? groupNames : undefined,
      });
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

  const updateItemState = async (id: number, state: 'unread' | 'read' | 'archived' | 'deleted') => {
    try {
      await invoke('update_item_state', { id, state });
      // Update local state
      const item = items.value.find(i => i.id === id);
      if (item) {
        item.state = state;
        // If marked as deleted, remove from local state immediately
        if (state === 'deleted') {
          items.value = items.value.filter(i => i.id !== id);
        }
      }
    } catch (e) {
      error.value = e as string;
      console.error('Failed to update item state:', e);
    }
  };

  const bulkUpdateItemState = async (ids: number[], state: 'unread' | 'read' | 'archived' | 'deleted') => {
    if (ids.length === 0) return;
    
    try {
      await invoke('bulk_update_item_state', { ids, state });
      // Update local state
      if (state === 'deleted') {
        // Remove deleted items from local state immediately (unless we're viewing trash)
        // Check if we're currently viewing deleted items by checking if any deleted items exist in the list
        const viewingDeleted = items.value.some(i => i.state === 'deleted');
        if (!viewingDeleted) {
          // Not viewing trash, so remove deleted items from view
          items.value = items.value.filter(i => !ids.includes(i.id));
        } else {
          // Viewing trash, update the state but keep items visible
          ids.forEach(id => {
            const item = items.value.find(i => i.id === id);
            if (item) {
              item.state = state;
            }
          });
        }
      } else {
        // Update state for other operations (including recovery from deleted)
        ids.forEach(id => {
          const item = items.value.find(i => i.id === id);
          if (item) {
            const wasDeleted = item.state === 'deleted';
            item.state = state;
            // If recovering from deleted, remove from trash view
            if (wasDeleted && state !== 'deleted') {
              items.value = items.value.filter(i => i.id !== id);
            }
          }
        });
      }
    } catch (e) {
      error.value = e as string;
      console.error('Failed to bulk update item state:', e);
      throw e;
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
    bulkUpdateItemState,
    unreadCount,
  };
}

