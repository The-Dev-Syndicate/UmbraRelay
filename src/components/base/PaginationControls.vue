<template>
  <div class="pagination-controls" v-if="totalPages > 1 || totalItems > 0">
    <div class="pagination-info">
      Showing {{ startItem }}-{{ endItem }} of {{ totalItems }} items
    </div>
    <div class="pagination-controls-right">
      <div class="items-per-page-selector">
        <label for="items-per-page-select">Items per page:</label>
        <select
          id="items-per-page-select"
          :value="itemsPerPage"
          @change="handleItemsPerPageChange"
          class="items-per-page-select"
        >
          <option v-for="option in itemsPerPageOptions" :key="option" :value="option">
            {{ option }}
          </option>
        </select>
      </div>
      <div class="pagination-buttons" v-if="totalPages > 1">
        <button
          @click="previousPage"
          :disabled="!hasPreviousPage"
          class="pagination-btn"
          title="Previous page"
        >
          ‹
        </button>
        
        <div class="page-numbers">
          <button
            v-for="page in visiblePages"
            :key="page"
            @click="goToPage(page)"
            :class="['page-btn', { active: page === currentPage }]"
          >
            {{ page }}
          </button>
        </div>
        
        <button
          @click="nextPage"
          :disabled="!hasNextPage"
          class="pagination-btn"
          title="Next page"
        >
          ›
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  currentPage: number;
  totalPages: number;
  totalItems: number;
  itemsPerPage: number;
  itemsPerPageOptions?: number[];
}>();

const emit = defineEmits<{
  (e: 'go-to-page', page: number): void;
  (e: 'next-page'): void;
  (e: 'previous-page'): void;
  (e: 'items-per-page-change', value: number): void;
}>();

const itemsPerPageOptions = computed(() => props.itemsPerPageOptions || [10, 20, 50, 100]);

const startItem = computed(() => {
  return props.totalItems === 0 ? 0 : (props.currentPage - 1) * props.itemsPerPage + 1;
});

const endItem = computed(() => {
  const end = props.currentPage * props.itemsPerPage;
  return Math.min(end, props.totalItems);
});

const hasPreviousPage = computed(() => props.currentPage > 1);
const hasNextPage = computed(() => props.currentPage < props.totalPages);

const visiblePages = computed(() => {
  const pages: number[] = [];
  const maxVisible = 5;
  let start = Math.max(1, props.currentPage - Math.floor(maxVisible / 2));
  let end = Math.min(props.totalPages, start + maxVisible - 1);
  
  // Adjust start if we're near the end
  if (end - start < maxVisible - 1) {
    start = Math.max(1, end - maxVisible + 1);
  }
  
  for (let i = start; i <= end; i++) {
    pages.push(i);
  }
  
  return pages;
});

const goToPage = (page: number) => {
  if (page >= 1 && page <= props.totalPages) {
    emit('go-to-page', page);
  }
};

const nextPage = () => {
  if (hasNextPage.value) {
    emit('next-page');
  }
};

const previousPage = () => {
  if (hasPreviousPage.value) {
    emit('previous-page');
  }
};

const handleItemsPerPageChange = (event: Event) => {
  const target = event.target as HTMLSelectElement;
  const value = parseInt(target.value, 10);
  emit('items-per-page-change', value);
};
</script>

<style scoped lang="scss">
@use '../../styles/variables' as *;

.pagination-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: $spacing-lg;
  margin-top: $spacing-lg;
  border-top: 1px solid var(--color-border-light);
  gap: $spacing-md;
  flex-wrap: wrap;
}

.pagination-controls-right {
  display: flex;
  align-items: center;
  gap: $spacing-lg;
  flex-wrap: wrap;
}

.items-per-page-selector {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  font-size: $font-sm;
  color: var(--color-text-secondary);

  label {
    white-space: nowrap;
  }
}

.items-per-page-select {
  padding: $spacing-xs $spacing-sm;
  border: 1px solid var(--color-border);
  border-radius: $radius-sm;
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  font-size: $font-sm;
  cursor: pointer;
  transition: all $transition-base;

  &:hover {
    border-color: var(--color-primary);
  }

  &:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px rgba(var(--color-primary-rgb, 90, 141, 232), 0.2);
  }
}

.pagination-info {
  color: var(--color-text-secondary);
  font-size: $font-sm;
}

.pagination-buttons {
  display: flex;
  align-items: center;
  gap: $spacing-xs;
}

.pagination-btn,
.page-btn {
  min-width: 32px;
  height: 32px;
  padding: 0 $spacing-sm;
  border: 1px solid var(--color-border);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  border-radius: $radius-sm;
  cursor: pointer;
  font-size: $font-md;
  transition: all $transition-base;
  display: flex;
  align-items: center;
  justify-content: center;

  &:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  &.active {
    background: var(--color-primary);
    color: var(--color-primary-text);
    border-color: var(--color-primary);
  }
}

.page-numbers {
  display: flex;
  gap: $spacing-xs;
}

@media (max-width: 768px) {
  .pagination-controls {
    flex-direction: column;
    align-items: stretch;
  }

  .pagination-info {
    text-align: center;
  }

  .pagination-controls-right {
    flex-direction: column;
    align-items: stretch;
    gap: $spacing-md;
  }

  .items-per-page-selector {
    justify-content: center;
  }

  .pagination-buttons {
    justify-content: center;
  }
}
</style>
