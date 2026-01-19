<template>
  <div class="secrets-manager">
    <div class="sticky-header">
      <div class="header-left">
        <h1>Secrets</h1>
      </div>
      <div class="header-actions">
        <button @click="showAddSecretModal = true" class="action-button primary">
          + Add Secret
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading">Loading secrets...</div>
    <div v-if="error" class="error-message">
      Error: {{ error }}
      <button @click="error = null" class="dismiss-error">×</button>
    </div>

    <div class="secrets-list">
      <div v-for="secret in secrets" :key="secret.id" class="secret-card">
        <div class="secret-header">
          <div>
            <h3>{{ secret.name }}</h3>
            <div class="secret-meta">
              <span class="ttl-type">{{ secret.ttl_type }}</span>
              <span v-if="secret.expires_at" class="expiry-info">
                Expires: {{ formatDate(secret.expires_at) }}
              </span>
              <span v-else class="expiry-info">Never expires</span>
            </div>
          </div>
          <div class="secret-actions">
            <button 
              @click="editSecret(secret)" 
              class="icon-button"
              title="Edit"
            >
              ✏️
            </button>
            <button 
              @click="removeSecret(secret.id)" 
              class="icon-button delete"
              :disabled="deletingSecrets.has(secret.id)"
              title="Delete"
            >
              🗑️
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Add/Edit Secret Modal -->
    <div v-if="showAddSecretModal || editingSecret" class="edit-panel-overlay" @click="closeSecretModal">
      <div class="edit-panel" @click.stop>
        <div class="edit-panel-header">
          <h2>{{ editingSecret ? 'Edit Secret' : 'Add Secret' }}</h2>
          <button @click="closeSecretModal" class="close-button" title="Close">×</button>
        </div>
        
        <div class="edit-panel-content">
          <form @submit.prevent="saveSecret" class="source-form" novalidate>
            <div class="form-group">
              <label>Name</label>
              <input v-model="secretForm.name" type="text" required placeholder="e.g., GitHub Token" />
            </div>
            <div class="form-group">
              <label>Value</label>
              <input 
                v-model="secretForm.value" 
                type="password" 
                :required="!editingSecret"
                placeholder="Enter secret value"
              />
              <small v-if="editingSecret" class="hint">Leave blank to keep existing value</small>
            </div>
            <div class="form-group">
              <label>Expiry Type</label>
              <select v-model="secretForm.ttlType" @change="onTtlTypeChange">
                <option value="forever">Forever (No expiration)</option>
                <option value="relative">Relative (e.g., 30 days)</option>
                <option value="absolute">Absolute Date</option>
              </select>
            </div>
            <div v-if="secretForm.ttlType === 'relative'" class="form-group">
              <label>Duration</label>
              <input 
                v-model="secretForm.ttlValue" 
                type="text" 
                placeholder="e.g., 30d, 1w, 6M, 1y"
              />
              <small class="hint">Format: number + unit (s, m, h, d, w, M, y)</small>
            </div>
            <div v-if="secretForm.ttlType === 'absolute'" class="form-group">
              <label>Expiry Date</label>
              <input 
                v-model="secretForm.ttlValue" 
                type="datetime-local"
              />
            </div>
            <div class="form-actions">
              <button type="button" @click="closeSecretModal" class="cancel-button">Cancel</button>
              <button type="submit" class="submit-button">
                {{ editingSecret ? 'Save Changes' : 'Create Secret' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Secret } from '../../types';
import { formatDate } from '../../utils/formatting';
import { ask, MessageDialogOptions } from '@tauri-apps/plugin-dialog';

const secrets = ref<Secret[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const showAddSecretModal = ref(false);
const editingSecret = ref<Secret | null>(null);
const deletingSecrets = ref<Set<number>>(new Set());

const secretForm = ref({
  name: '',
  value: '',
  ttlType: 'forever' as 'forever' | 'relative' | 'absolute',
  ttlValue: '',
});

const fetchSecrets = async () => {
  loading.value = true;
  error.value = null;
  try {
    secrets.value = await invoke<Secret[]>('get_secrets');
  } catch (e) {
    error.value = e as string;
    console.error('Failed to fetch secrets:', e);
  } finally {
    loading.value = false;
  }
};

const saveSecret = async () => {
  if (!secretForm.value.name.trim()) {
    alert('Please enter a secret name');
    return;
  }
  
  if (!editingSecret.value && !secretForm.value.value.trim()) {
    alert('Please enter a secret value');
    return;
  }
  
  try {
    if (editingSecret.value) {
      await invoke('update_secret', {
        id: editingSecret.value.id,
        name: secretForm.value.name,
        value: secretForm.value.value || undefined,
        ttl_type: secretForm.value.ttlType,
        ttl_value: secretForm.value.ttlValue || undefined,
      });
    } else {
      await invoke('create_secret', {
        name: secretForm.value.name,
        value: secretForm.value.value,
        ttl_type: secretForm.value.ttlType,
        ttl_value: secretForm.value.ttlValue || undefined,
      });
    }
    
    await fetchSecrets();
    closeSecretModal();
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to save secret: ${errorMsg}`);
    error.value = errorMsg;
  }
};

const editSecret = (secret: Secret) => {
  editingSecret.value = secret;
  secretForm.value = {
    name: secret.name,
    value: '', // Don't show existing value
    ttlType: secret.ttl_type as 'forever' | 'relative' | 'absolute',
    ttlValue: secret.ttl_value || '',
  };
};

const removeSecret = async (id: number) => {
  const secret = secrets.value.find(s => s.id === id);
  if (!secret) return;
  
  const confirmed = await ask(
    `Are you sure you want to delete the secret "${secret.name}"?`,
    {
      title: 'Delete Secret',
      kind: 'warning',
      okLabel: 'Confirm',
      cancelLabel: 'Cancel',
    } as MessageDialogOptions
  );
  
  if (!confirmed) return;
  
  deletingSecrets.value.add(id);
  try {
    await invoke('delete_secret', { id });
    await fetchSecrets();
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to delete secret: ${errorMsg}`);
    error.value = errorMsg;
  } finally {
    deletingSecrets.value.delete(id);
  }
};

const closeSecretModal = () => {
  showAddSecretModal.value = false;
  editingSecret.value = null;
  secretForm.value = {
    name: '',
    value: '',
    ttlType: 'forever',
    ttlValue: '',
  };
};

const onTtlTypeChange = () => {
  if (secretForm.value.ttlType === 'forever') {
    secretForm.value.ttlValue = '';
  }
};

onMounted(async () => {
  await fetchSecrets();
});
</script>

<style scoped>
.secrets-manager {
  padding: 20px;
}

.secret-card {
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 12px;
}

.secret-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.secret-meta {
  display: flex;
  gap: 12px;
  margin-top: 8px;
  font-size: 14px;
  color: var(--text-secondary);
}

.ttl-type {
  text-transform: capitalize;
  font-weight: 500;
}

.expiry-info {
  color: var(--text-secondary);
}

.secret-actions {
  display: flex;
  gap: 8px;
}
</style>

