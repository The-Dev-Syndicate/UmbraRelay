<template>
  <div class="view-config-overlay" @click="$emit('close')">
    <div class="view-config-panel" @click.stop>
      <div class="view-config-header">
        <h2>{{ editingViewId ? 'Edit Custom View' : 'Create Custom View' }}</h2>
        <button @click="$emit('close')" class="close-button" title="Close">×</button>
      </div>
      
      <div v-if="error" class="error-message">
        {{ error }}
        <button @click="error = null" class="dismiss-error">×</button>
      </div>

      <div class="view-config-content">
        <form @submit.prevent="saveView" class="view-form">
          <div class="form-group">
            <label>View Name</label>
            <input 
              v-model="form.name" 
              type="text" 
              required 
              placeholder="e.g., Work View" 
              :disabled="saving"
            />
          </div>

          <div class="form-group">
            <label>Sources (optional)</label>
            <p class="form-help">Select specific sources to filter by. Leave empty to include all sources.</p>
            <div class="checkbox-group">
              <label class="checkbox-option">
                <input 
                  type="checkbox" 
                  :checked="allSourcesSelected"
                  @change="toggleAllSources"
                />
                <span>Select All</span>
              </label>
              <label 
                v-for="source in sources" 
                :key="source.id"
                class="checkbox-option"
              >
                <input 
                  type="checkbox" 
                  :value="source.id"
                  v-model="form.sourceIds"
                  :disabled="saving"
                />
                <span>{{ source.name }}</span>
              </label>
            </div>
          </div>

          <div class="form-group">
            <label>Groups (optional)</label>
            <p class="form-help">Select specific groups to filter by. Leave empty to include all groups.</p>
            <div class="checkbox-group">
              <label class="checkbox-option">
                <input 
                  type="checkbox" 
                  :checked="allGroupsSelected"
                  @change="toggleAllGroups"
                />
                <span>Select All</span>
              </label>
              <label 
                v-for="group in availableGroups" 
                :key="group"
                class="checkbox-option"
              >
                <input 
                  type="checkbox" 
                  :value="group"
                  v-model="form.groupNames"
                  :disabled="saving"
                />
                <span>{{ group }}</span>
              </label>
            </div>
          </div>

          <div class="form-actions">
            <button type="button" @click="$emit('close')" class="cancel-button" :disabled="saving">
              Cancel
            </button>
            <button type="submit" class="save-button" :disabled="saving || !form.name.trim()">
              {{ saving ? 'Saving...' : (editingViewId ? 'Update' : 'Create') }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useCustomViews } from '../composables/useCustomViews';
import { useSources } from '../composables/useSources';
import type { CustomViewInput } from '../types';

const props = defineProps<{
  viewId?: number | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saved'): void;
}>();

const { createCustomView, updateCustomView, getCustomView, error: viewError } = useCustomViews();
const { sources, fetchSources } = useSources();

const form = ref<CustomViewInput>({
  name: '',
  sourceIds: [],
  groupNames: [],
});

const saving = ref(false);
const error = ref<string | null>(null);
const editingViewId = computed(() => props.viewId);

// Get available groups from sources
const availableGroups = computed(() => {
  const groups = new Set<string>();
  sources.value.forEach(source => {
    if (source.group) {
      const parsed = source.group.split(',').map(g => g.trim()).filter(g => g.length > 0);
      parsed.forEach(g => groups.add(g));
    }
  });
  return Array.from(groups).sort();
});

const allSourcesSelected = computed(() => {
  return sources.value.length > 0 && form.value.sourceIds?.length === sources.value.length;
});

const allGroupsSelected = computed(() => {
  return availableGroups.value.length > 0 && form.value.groupNames?.length === availableGroups.value.length;
});

const toggleAllSources = () => {
  if (allSourcesSelected.value) {
    form.value.sourceIds = [];
  } else {
    form.value.sourceIds = sources.value.map(s => s.id);
  }
};

const toggleAllGroups = () => {
  if (allGroupsSelected.value) {
    form.value.groupNames = [];
  } else {
    form.value.groupNames = [...availableGroups.value];
  }
};

const loadView = async () => {
  if (editingViewId.value) {
    const view = await getCustomView(editingViewId.value);
    if (view) {
      form.value.name = view.name;
      
      // Parse source_ids JSON
      if (view.source_ids) {
        try {
          form.value.sourceIds = JSON.parse(view.source_ids);
        } catch {
          form.value.sourceIds = [];
        }
      } else {
        form.value.sourceIds = [];
      }
      
      // Parse group_names JSON
      if (view.group_names) {
        try {
          form.value.groupNames = JSON.parse(view.group_names);
        } catch {
          form.value.groupNames = [];
        }
      } else {
        form.value.groupNames = [];
      }
    }
  } else {
    form.value = {
      name: '',
      sourceIds: [],
      groupNames: [],
    };
  }
};

const saveView = async () => {
  saving.value = true;
  error.value = null;
  
  try {
    const input: CustomViewInput = {
      name: form.value.name.trim(),
      sourceIds: form.value.sourceIds && form.value.sourceIds.length > 0 ? form.value.sourceIds : null,
      groupNames: form.value.groupNames && form.value.groupNames.length > 0 ? form.value.groupNames : null,
    };
    
    if (editingViewId.value) {
      await updateCustomView(editingViewId.value, input);
    } else {
      await createCustomView(input);
    }
    
    emit('saved');
  } catch (e) {
    error.value = e as string || 'Failed to save view';
  } finally {
    saving.value = false;
  }
};

watch(() => props.viewId, () => {
  loadView();
}, { immediate: true });

watch(viewError, (newError) => {
  if (newError) {
    error.value = newError;
  }
});

onMounted(() => {
  fetchSources();
  loadView();
});
</script>

<style scoped>
.view-config-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
  align-items: stretch;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.view-config-panel {
  width: 500px;
  max-width: 90vw;
  background: white;
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
  animation: slideIn 0.3s ease;
  overflow-y: auto;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}

.view-config-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #e0e0e0;
  background: #f8f9fa;
}

.view-config-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #1a1a1a;
}

.close-button {
  background: none;
  border: none;
  font-size: 28px;
  color: #666;
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-button:hover {
  background: #e0e0e0;
  color: #333;
}

.view-config-content {
  padding: 20px;
  flex: 1;
}

.error-message {
  background: #fee;
  color: #c33;
  padding: 12px 16px;
  margin: 20px;
  border-radius: 4px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.dismiss-error {
  background: none;
  border: none;
  color: #c33;
  cursor: pointer;
  font-size: 20px;
  padding: 0;
  width: 24px;
  height: 24px;
}

.view-form {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-weight: 600;
  color: #333;
  font-size: 14px;
}

.form-help {
  font-size: 12px;
  color: #666;
  margin: 0;
  font-style: italic;
}

.form-group input[type="text"] {
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.form-group input[type="text"]:focus {
  outline: none;
  border-color: #396cd8;
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 300px;
  overflow-y: auto;
  padding: 8px;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  background: #fafafa;
}

.checkbox-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.2s;
}

.checkbox-option:hover {
  background: #f0f0f0;
}

.checkbox-option input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
  accent-color: #396cd8;
}

.checkbox-option span {
  font-size: 14px;
  color: #333;
  user-select: none;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 20px;
  border-top: 1px solid #e0e0e0;
}

.cancel-button {
  padding: 10px 20px;
  border: 1px solid #ddd;
  background: white;
  color: #333;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.cancel-button:hover:not(:disabled) {
  background: #f5f5f5;
  border-color: #bbb;
}

.cancel-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.save-button {
  padding: 10px 20px;
  border: none;
  background: #396cd8;
  color: white;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.2s;
}

.save-button:hover:not(:disabled) {
  background: #2952b8;
}

.save-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>

