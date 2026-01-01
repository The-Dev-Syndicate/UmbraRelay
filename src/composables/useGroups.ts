import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Group } from '../types';

const groups = ref<Group[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

export function useGroups() {
  const fetchGroups = async () => {
    loading.value = true;
    error.value = null;
    try {
      groups.value = await invoke<Group[]>('get_groups');
    } catch (e) {
      error.value = e as string;
      console.error('Failed to fetch groups:', e);
    } finally {
      loading.value = false;
    }
  };

  const addGroup = async (name: string): Promise<number> => {
    try {
      const id = await invoke<number>('add_group', { name });
      await fetchGroups();
      return id;
    } catch (e) {
      error.value = e as string;
      console.error('Failed to add group:', e);
      throw e; // Re-throw so caller can handle it
    }
  };

  const updateGroup = async (id: number, name: string) => {
    try {
      await invoke('update_group', { id, name });
      await fetchGroups();
    } catch (e) {
      error.value = e as string;
      console.error('Failed to update group:', e);
      throw e; // Re-throw so UI can handle it
    }
  };

  const removeGroup = async (id: number) => {
    try {
      await invoke('remove_group', { id });
      await fetchGroups();
    } catch (e) {
      error.value = e as string;
      console.error('Failed to remove group:', e);
      throw e; // Re-throw so UI can handle it
    }
  };

  return {
    groups,
    loading,
    error,
    fetchGroups,
    addGroup,
    updateGroup,
    removeGroup,
  };
}

