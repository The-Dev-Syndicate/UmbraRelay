<template>
  <div class="inbox-view">
    <div class="header">
      <h1>Inbox</h1>
      <div class="filters">
        <button
          v-for="filter in filters"
          :key="filter"
          :class="{ active: currentFilter === filter }"
          @click="setFilter(filter)"
        >
          {{ filter }}
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading">Loading...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <div v-else-if="filteredItems.length === 0" class="empty">
      No items found.
    </div>
    <div v-else class="items-list">
      <div
        v-for="item in filteredItems"
        :key="item.id"
        class="item-card"
        :class="item.state"
        @click="selectItem(item.id)"
      >
        <div class="item-header">
          <span class="item-type">{{ item.item_type }}</span>
          <span class="item-state">{{ item.state }}</span>
        </div>
        <h3 class="item-title">{{ item.title }}</h3>
        <p v-if="item.summary" class="item-summary">{{ truncate(item.summary, 150) }}</p>
        <div class="item-footer">
          <span class="item-date">{{ formatDate(item.created_at) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useItems } from '../composables/useItems';

const emit = defineEmits<{
  (e: 'select-item', id: number): void;
}>();

const { items, loading, error, fetchItems } = useItems();
const currentFilter = ref<string | null>(null);

const filters = ['All', 'Unread', 'Read', 'Archived'];

const filteredItems = computed(() => {
  if (!currentFilter.value || currentFilter.value === 'all') {
    return items.value;
  }
  return items.value.filter(item => item.state === currentFilter.value);
});

const setFilter = (filter: string) => {
  const filterValue = filter === 'All' ? null : filter.toLowerCase();
  currentFilter.value = filterValue;
  fetchItems(filterValue || undefined);
};

const selectItem = (id: number) => {
  emit('select-item', id);
};

const truncate = (text: string, length: number) => {
  if (text.length <= length) return text;
  return text.substring(0, length) + '...';
};

const formatDate = (timestamp: number) => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
};

onMounted(() => {
  fetchItems();
});
</script>

<style scoped>
.inbox-view {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header h1 {
  margin: 0;
}

.filters {
  display: flex;
  gap: 10px;
}

.filters button {
  padding: 8px 16px;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
  border-radius: 4px;
}

.filters button.active {
  background: #396cd8;
  color: white;
  border-color: #396cd8;
}

.loading, .error, .empty {
  text-align: center;
  padding: 40px;
  color: #666;
}

.error {
  color: #d32f2f;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.item-card {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.2s;
  background: white;
}

.item-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.item-card.unread {
  border-left: 4px solid #396cd8;
}

.item-card.read {
  opacity: 0.7;
}

.item-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 12px;
  text-transform: uppercase;
}

.item-type {
  color: #666;
}

.item-state {
  color: #396cd8;
  font-weight: bold;
}

.item-title {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
}

.item-summary {
  margin: 0 0 8px 0;
  color: #666;
  line-height: 1.5;
}

.item-footer {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #999;
  margin-top: 8px;
}
</style>

