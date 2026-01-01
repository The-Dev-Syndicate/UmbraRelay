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

  const addSource = async (source: SourceInput): Promise<number> => {
    try {
      const id = await invoke<number>('add_source', { source });
      await fetchSources();
      return id;
    } catch (e) {
      error.value = e as string;
      console.error('Failed to add source:', e);
      throw e; // Re-throw so UI can handle it
    }
  };

  const updateSource = async (id: number, update: UpdateSourceInput) => {
    try {
      // Ensure group_ids is properly serialized (convert Proxy to plain array if needed)
      const serializedUpdate: UpdateSourceInput = {
        name: update.name,
        config_json: update.config_json,
        enabled: update.enabled,
        token: update.token,
        group_ids: update.group_ids ? (Array.isArray(update.group_ids) ? [...update.group_ids] : update.group_ids) : undefined,
      };
      
      await invoke('update_source', { id, update: serializedUpdate });
      await fetchSources();
    } catch (e) {
      error.value = e as string;
      console.error('Failed to update source:', e);
      throw e; // Re-throw so UI can handle it
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
      throw e; // Re-throw so UI can handle it
    }
  };

  const syncAllSources = async () => {
    try {
      await invoke('sync_all_sources');
      await fetchSources();
    } catch (e) {
      error.value = e as string;
      console.error('Failed to sync all sources:', e);
      throw e; // Re-throw so UI can handle it
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
    syncAllSources,
  };
}

