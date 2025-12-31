import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Source, SourceInput, UpdateSourceInput } from '../types';

const sources = ref<Source[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

export function useSources() {
  const fetchSources = async () => {
    loading.value = true;
    error.value = null;
    try {
      sources.value = await invoke<Source[]>('get_sources');
    } catch (e) {
      error.value = e as string;
      console.error('Failed to fetch sources:', e);
    } finally {
      loading.value = false;
    }
  };

  const addSource = async (source: SourceInput): Promise<number | null> => {
    try {
      const id = await invoke<number>('add_source', { source });
      await fetchSources();
      return id;
    } catch (e) {
      error.value = e as string;
      console.error('Failed to add source:', e);
      return null;
    }
  };

  const updateSource = async (id: number, update: UpdateSourceInput) => {
    try {
      await invoke('update_source', { id, update });
      await fetchSources();
    } catch (e) {
      error.value = e as string;
      console.error('Failed to update source:', e);
    }
  };

  const removeSource = async (id: number) => {
    try {
      console.log('Removing source:', id);
      await invoke('remove_source', { id });
      console.log('Source removed successfully, refreshing list...');
      await fetchSources();
      console.log('Sources list refreshed');
    } catch (e) {
      const errorMsg = e as string;
      error.value = errorMsg;
      console.error('Failed to remove source:', e);
      throw e; // Re-throw so UI can handle it
    }
  };

  const syncSource = async (id: number) => {
    try {
      await invoke('sync_source', { id });
      await fetchSources();
    } catch (e) {
      error.value = e as string;
      console.error('Failed to sync source:', e);
    }
  };

  return {
    sources,
    loading,
    error,
    fetchSources,
    addSource,
    updateSource,
    removeSource,
    syncSource,
  };
}

