<template>
  <div class="leaving-soon-view">
    <div class="header">
      <h1>Leaving Soon</h1>
      <p class="subtitle">Items that will be deleted in 7 days or less</p>
    </div>

    <div v-if="loading" class="loading">Loading...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <div v-else-if="filteredItems.length === 0" class="empty">
      No items leaving soon.
    </div>
    <div v-else class="items-list">
      <div
        v-for="item in filteredItems"
        :key="item.id"
        class="item-card"
        :class="item.state"
        @click="selectItem(item.id)"
      >
        <div class="item-card-header">
          <span class="item-source-name">{{ item.source_name || 'Unknown Source' }}</span>
          <div class="item-badges">
            <span class="item-type-badge">{{ item.item_type.toUpperCase() }}</span>
            <span class="item-state-badge" :class="item.state">{{ item.state.toUpperCase() }}</span>
          </div>
        </div>
        <div class="item-content">
          <div class="item-content-wrapper">
            <div v-if="item.image_url" class="item-image-container">
              <img :src="item.image_url" :alt="item.title" class="item-image" />
            </div>
            <div class="item-text-content">
              <h3 class="item-title">{{ item.title }}</h3>
              <p v-if="item.summary && stripHtml(item.summary).trim() && stripHtml(item.summary).toLowerCase() !== 'comments'" class="item-summary">{{ truncate(stripHtml(item.summary), 200) }}</p>
            </div>
          </div>
        </div>
        <div class="item-footer">
          <span class="item-date">{{ formatDate(item.created_at) }}</span>
          <span 
            class="days-until-removal"
            :class="`days-${daysUntilRemoval(item)}`"
          >
            {{ daysUntilRemoval(item) }} day{{ daysUntilRemoval(item) !== 1 ? 's' : '' }} left
          </span>
          <div v-if="item.source_group" class="item-groups">
            <span 
              v-for="group in parseGroups(item.source_group)" 
              :key="group" 
              class="group-tag"
            >
              {{ group }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useItems } from '../../composables/useItems';
import type { Item } from '../../types';
import { formatDate, truncate, stripHtml, parseGroups } from '../../utils/formatting';

const emit = defineEmits<{
  (e: 'select-item', id: number): void;
}>();

const { items, loading, error, fetchItems, updateItemState } = useItems();

// Calculate days until removal (30 days from created_at)
const daysUntilRemoval = (item: Item): number => {
  const now = Math.floor(Date.now() / 1000);
  const itemAge = now - item.created_at;
  const daysOld = Math.floor(itemAge / (24 * 60 * 60));
  const daysLeft = 30 - daysOld;
  return Math.max(0, daysLeft);
};

// Filter items that are 23+ days old (7 days or less until deletion) and not archived
const filteredItems = computed(() => {
  const now = Math.floor(Date.now() / 1000);
  
  return items.value
    .filter(item => {
      // Exclude archived items (they never get deleted)
      if (item.state === 'archived') return false;
      
      // Calculate days old
      const itemAge = now - item.created_at;
      const daysOld = Math.floor(itemAge / (24 * 60 * 60));
      
      // Show items that are 23+ days old (7 days or less until deletion)
      return daysOld >= 23 && daysOld < 30;
    })
    .sort((a, b) => {
      // Sort by oldest first (reverse date order)
      return a.created_at - b.created_at;
    });
});

const selectItem = async (id: number) => {
  // Mark item as read if it's currently unread
  const item = items.value.find(i => i.id === id);
  if (item && item.state === 'unread') {
    await updateItemState(id, 'read');
  }
  emit('select-item', id);
};


onMounted(() => {
  fetchItems();
});
</script>


