import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const DEFAULT_ITEMS_PER_PAGE = 10;
const ITEMS_PER_PAGE_OPTIONS = [10, 20, 50, 100];

export function usePagination<T>(items: () => T[], defaultItemsPerPage?: number) {
  const currentPage = ref(1);
  const itemsPerPage = ref(defaultItemsPerPage || DEFAULT_ITEMS_PER_PAGE);

  // Load items per page preference
  const loadItemsPerPagePreference = async () => {
    try {
      const saved = await invoke<string | null>('get_user_preference', { key: 'items_per_page' });
      if (saved) {
        const parsed = parseInt(saved, 10);
        if (ITEMS_PER_PAGE_OPTIONS.includes(parsed)) {
          itemsPerPage.value = parsed;
        }
      }
    } catch (error) {
      console.error('Failed to load items per page preference:', error);
    }
  };

  // Save items per page preference
  const saveItemsPerPagePreference = async (value: number) => {
    try {
      await invoke('set_user_preference', { key: 'items_per_page', value: value.toString() });
    } catch (error) {
      console.error('Failed to save items per page preference:', error);
    }
  };

  // Load preference on initialization
  loadItemsPerPagePreference();

  // Watch for itemsPerPage changes and save preference
  watch(itemsPerPage, (newValue) => {
    saveItemsPerPagePreference(newValue);
    resetPage(); // Reset to page 1 when changing items per page
  });

  const totalItems = computed(() => items().length);
  const totalPages = computed(() => Math.ceil(totalItems.value / itemsPerPage.value));
  
  const paginatedItems = computed(() => {
    const start = (currentPage.value - 1) * itemsPerPage.value;
    const end = start + itemsPerPage.value;
    return items().slice(start, end);
  });

  const hasPreviousPage = computed(() => currentPage.value > 1);
  const hasNextPage = computed(() => currentPage.value < totalPages.value);

  const goToPage = (page: number) => {
    if (page >= 1 && page <= totalPages.value) {
      currentPage.value = page;
      // Scroll to top when changing pages
      window.scrollTo({ top: 0, behavior: 'smooth' });
    }
  };

  const nextPage = () => {
    if (hasNextPage.value) {
      goToPage(currentPage.value + 1);
    }
  };

  const previousPage = () => {
    if (hasPreviousPage.value) {
      goToPage(currentPage.value - 1);
    }
  };

  const resetPage = () => {
    currentPage.value = 1;
  };

  const setItemsPerPage = (value: number) => {
    if (ITEMS_PER_PAGE_OPTIONS.includes(value)) {
      itemsPerPage.value = value;
    }
  };

  // Watch for changes in total items and reset to page 1 if current page is out of bounds
  const checkPageBounds = () => {
    if (currentPage.value > totalPages.value && totalPages.value > 0) {
      currentPage.value = totalPages.value;
    } else if (currentPage.value < 1 && totalItems.value > 0) {
      currentPage.value = 1;
    }
  };

  return {
    currentPage,
    totalPages,
    totalItems,
    paginatedItems,
    hasPreviousPage,
    hasNextPage,
    itemsPerPage,
    itemsPerPageOptions: ITEMS_PER_PAGE_OPTIONS,
    goToPage,
    nextPage,
    previousPage,
    resetPage,
    setItemsPerPage,
    checkPageBounds,
  };
}
