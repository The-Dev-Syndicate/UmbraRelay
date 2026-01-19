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
import { useCustomViews } from '../../composables/useCustomViews';
import { useSources } from '../../composables/useSources';
import { useGroups } from '../../composables/useGroups';
import type { CustomViewInput } from '../../types';

const props = defineProps<{
  viewId?: number | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saved'): void;
}>();

const { createCustomView, updateCustomView, getCustomView, error: viewError } = useCustomViews();
const { sources, fetchSources } = useSources();
const { groups, fetchGroups } = useGroups();

const form = ref<CustomViewInput>({
  name: '',
  sourceIds: [],
  groupNames: [],
});

const saving = ref(false);
const error = ref<string | null>(null);
const editingViewId = computed(() => props.viewId);

// Get available groups from groups list
const availableGroups = computed(() => {
  return groups.value.map(g => g.name).sort();
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
  fetchGroups();
  loadView();
});
</script>


