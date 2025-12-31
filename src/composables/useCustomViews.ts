import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { CustomView, CustomViewInput } from '../types';

const customViews = ref<CustomView[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

export function useCustomViews() {
  const fetchCustomViews = async () => {
    loading.value = true;
    error.value = null;
    try {
      customViews.value = await invoke<CustomView[]>('get_custom_views');
    } catch (e) {
      error.value = e as string;
      console.error('Failed to fetch custom views:', e);
    } finally {
      loading.value = false;
    }
  };

  const createCustomView = async (input: CustomViewInput) => {
    loading.value = true;
    error.value = null;
    try {
      const sourceIdsJson = input.sourceIds && input.sourceIds.length > 0
        ? JSON.stringify(input.sourceIds)
        : null;
      const groupNamesJson = input.groupNames && input.groupNames.length > 0
        ? JSON.stringify(input.groupNames)
        : null;
      
      const id = await invoke<number>('add_custom_view', {
        name: input.name,
        sourceIds: sourceIdsJson,
        groupNames: groupNamesJson,
      });
      
      await fetchCustomViews();
      return id;
    } catch (e) {
      error.value = e as string;
      console.error('Failed to create custom view:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const updateCustomView = async (id: number, input: CustomViewInput) => {
    loading.value = true;
    error.value = null;
    try {
      const sourceIdsJson = input.sourceIds && input.sourceIds.length > 0
        ? JSON.stringify(input.sourceIds)
        : null;
      const groupNamesJson = input.groupNames && input.groupNames.length > 0
        ? JSON.stringify(input.groupNames)
        : null;
      
      await invoke('update_custom_view', {
        id,
        name: input.name,
        sourceIds: sourceIdsJson,
        groupNames: groupNamesJson,
      });
      
      await fetchCustomViews();
    } catch (e) {
      error.value = e as string;
      console.error('Failed to update custom view:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const deleteCustomView = async (id: number) => {
    loading.value = true;
    error.value = null;
    try {
      await invoke('remove_custom_view', { id });
      await fetchCustomViews();
    } catch (e) {
      error.value = e as string;
      console.error('Failed to delete custom view:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  };

  const getCustomView = async (id: number): Promise<CustomView | null> => {
    try {
      return await invoke<CustomView>('get_custom_view', { id });
    } catch (e) {
      error.value = e as string;
      console.error('Failed to get custom view:', e);
      return null;
    }
  };

  return {
    customViews,
    loading,
    error,
    fetchCustomViews,
    createCustomView,
    updateCustomView,
    deleteCustomView,
    getCustomView,
  };
}

