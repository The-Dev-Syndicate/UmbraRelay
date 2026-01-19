<template>
  <div>
    <!-- Secrets Section -->
    <SettingsSection title="Security & Secrets">
      <template #actions>
        <button @click="showAddSecretModal = true" class="action-button primary">
          + Add Secret
        </button>
      </template>

      <div v-if="secretsLoading" class="loading">Loading secrets...</div>
      <div v-if="secretsError" class="error-message">
        Error: {{ secretsError }}
        <button @click="secretsError = null" class="dismiss-error">×</button>
      </div>
      
      <div v-if="!secretsLoading && secrets.length === 0" class="empty-state">
        <p>No secrets yet. Create your first secret to store API tokens.</p>
      </div>
      
      <div v-else-if="!secretsLoading" class="secrets-list">
        <div v-for="secret in secrets" :key="secret.id" class="secret-card" :class="getSecretHealthClass(secret)">
          <div class="secret-header">
            <div>
              <div style="display: flex; align-items: center; gap: 8px;">
                <h3>{{ secret.name }}</h3>
                <span v-if="getSecretHealthStatus(secret) === 'error'" class="token-status-badge error" title="Token has issues">
                  ⚠️ Token Issues
                </span>
                <span v-else-if="getSecretHealthStatus(secret) === 'warning'" class="token-status-badge warning" title="Token may have issues">
                  ⚠️ Warning
                </span>
                <span v-else-if="getSecretHealthStatus(secret) === 'testing'" class="token-status-badge testing" title="Testing token...">
                  🔄 Testing...
                </span>
                <span v-else-if="getSecretHealthStatus(secret) === 'good'" class="token-status-badge good" title="Token is valid">
                  ✓ Valid
                </span>
              </div>
              <div class="secret-meta">
                <span class="ttl-type">{{ secret.ttl_type }}</span>
                <span v-if="secret.expires_at" class="expiry-info">
                  Expires: {{ formatDate(secret.expires_at) }}
                </span>
                <span v-else class="expiry-info">Never expires</span>
                <span class="secret-count">
                  {{ getSecretSourceCount(secret.id) }} source{{ getSecretSourceCount(secret.id) !== 1 ? 's' : '' }}
                </span>
                <span v-if="secret.refresh_failure_count && secret.refresh_failure_count > 0" class="failure-count" style="color: #d32f2f; font-weight: 500;">
                  ({{ secret.refresh_failure_count }} refresh failure{{ secret.refresh_failure_count !== 1 ? 's' : '' }})
                </span>
              </div>
            </div>
            <div class="secret-actions">
              <button 
                @click="testSecretToken(secret.id)" 
                class="icon-button"
                :disabled="testingSecrets.has(secret.id)"
                :title="testingSecrets.has(secret.id) ? 'Testing...' : 'Test Token'"
              >
                {{ testingSecrets.has(secret.id) ? '🔄' : '🧪' }}
              </button>
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
    </SettingsSection>

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
              <small v-if="!editingSecret" class="hint">
                For GitHub tokens, expiration will be automatically detected if possible.
              </small>
            </div>
            <div class="form-group">
              <label>Expiry Type</label>
              <select v-model="secretForm.ttlType" @change="onTtlTypeChange">
                <option value="forever">Forever (No expiration)</option>
                <option value="relative">Relative (e.g., 30 days)</option>
                <option value="absolute">Absolute Date</option>
              </select>
              <small class="hint">If "Forever" is selected, we'll try to detect expiration from the token.</small>
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
import { ask, MessageDialogOptions } from '@tauri-apps/plugin-dialog';
import SettingsSection from '../base/SettingsSection.vue';
import { formatDate } from '../../utils/formatting';

const secrets = ref<any[]>([]);
const secretsLoading = ref(false);
const secretsError = ref<string | null>(null);
const showAddSecretModal = ref(false);
const editingSecret = ref<any | null>(null);
const deletingSecrets = ref<Set<number>>(new Set());
const testingSecrets = ref<Set<number>>(new Set());
const secretHealthStatus = ref<Map<number, 'good' | 'warning' | 'error' | 'testing'>>(new Map());
const sourceSecretMap = ref<Map<number, number>>(new Map()); // Maps source_id -> secret_id

const secretForm = ref({
  name: '',
  value: '',
  ttlType: 'forever' as 'forever' | 'relative' | 'absolute',
  ttlValue: '',
});

// Get count of sources using a secret
const getSecretSourceCount = (secretId: number): number => {
  let count = 0;
  for (const [, mappedSecretId] of sourceSecretMap.value.entries()) {
    if (mappedSecretId === secretId) {
      count++;
    }
  }
  return count;
};

// Build source -> secret mapping
const buildSourceSecretMap = async () => {
  sourceSecretMap.value.clear();
  const tauriCore = await import('@tauri-apps/api/core');
  if (!tauriCore?.invoke) {
    console.warn('Tauri API not available, skipping secret map build');
    return;
  }
  try {
    const { invoke } = tauriCore;
    const sources = await invoke<any[]>('get_sources');
    for (const source of sources) {
      try {
        const secretId = await invoke<number | null>('get_source_secret_id', { id: source.id });
        if (secretId !== null) {
          sourceSecretMap.value.set(source.id, secretId);
        }
      } catch (e) {
        // Ignore errors - source might not have a secret
        console.warn(`Failed to get secret_id for source ${source.id}:`, e);
      }
    }
  } catch (e) {
    console.warn('Failed to build source secret map:', e);
  }
};

// Secrets management functions
const fetchSecrets = async () => {
  secretsLoading.value = true;
  secretsError.value = null;
  try {
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore?.invoke) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    secrets.value = await tauriCore.invoke<any[]>('get_secrets');
  } catch (e) {
    secretsError.value = e instanceof Error ? e.message : String(e);
    console.error('Failed to fetch secrets:', e);
  } finally {
    secretsLoading.value = false;
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
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore?.invoke) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    const { invoke } = tauriCore;
    
    // If creating a new secret and ttlType is "forever", try to detect expiration from token
    let ttlType = secretForm.value.ttlType;
    let ttlValue = secretForm.value.ttlValue || undefined;
    
    if (!editingSecret.value && ttlType === 'forever' && secretForm.value.value.trim()) {
      // Try to detect expiration from GitHub token
      try {
        const detected = await invoke<{ ttl_type: string; ttl_value?: string } | null>('detect_github_token_expiration', {
          token: secretForm.value.value
        });
        if (detected) {
          ttlType = detected.ttl_type as 'forever' | 'relative' | 'absolute';
          ttlValue = detected.ttl_value;
        }
      } catch (e) {
        // If detection fails, continue with "forever"
        console.warn('Failed to detect token expiration:', e);
      }
    }
    
    if (editingSecret.value) {
      await invoke('update_secret', {
        id: editingSecret.value.id,
        name: secretForm.value.name,
        value: secretForm.value.value || undefined,
        ttl_type: ttlType,
        ttl_value: ttlValue,
      });
    } else {
      await invoke('create_secret', {
        name: secretForm.value.name,
        value: secretForm.value.value,
        ttl_type: ttlType,
        ttl_value: ttlValue,
        refresh_token: null,
      });
    }
    
    await fetchSecrets();
    await buildSourceSecretMap();
    closeSecretModal();
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to save secret: ${errorMsg}`);
    secretsError.value = errorMsg;
  }
};

const editSecret = (secret: any) => {
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
  
  const sourceCount = getSecretSourceCount(id);
  const confirmed = await ask(
    `Are you sure you want to delete the secret "${secret.name}"?${sourceCount > 0 ? ` This secret is used by ${sourceCount} source${sourceCount !== 1 ? 's' : ''}.` : ''}`,
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
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore?.invoke) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    await tauriCore.invoke('delete_secret', { id });
    await fetchSecrets();
    await buildSourceSecretMap();
    secretHealthStatus.value.delete(id);
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to delete secret: ${errorMsg}`);
    secretsError.value = errorMsg;
  } finally {
    deletingSecrets.value.delete(id);
  }
};

// Get secret health status
const getSecretHealthStatus = (secret: any): 'good' | 'warning' | 'error' | 'testing' => {
  if (testingSecrets.value.has(secret.id)) {
    return 'testing';
  }
  
  // Check if we have a cached status
  if (secretHealthStatus.value.has(secret.id)) {
    return secretHealthStatus.value.get(secret.id)!;
  }
  
  // Check refresh failure count
  if (secret.refresh_failure_count && secret.refresh_failure_count >= 3) {
    return 'error';
  }
  
  if (secret.refresh_failure_count && secret.refresh_failure_count > 0) {
    return 'warning';
  }
  
  // Check if expired
  if (secret.expires_at && secret.expires_at < Date.now() / 1000) {
    return 'error';
  }
  
  // Check if expiring soon (within 7 days)
  if (secret.expires_at) {
    const daysUntilExpiry = (secret.expires_at - Date.now() / 1000) / (24 * 60 * 60);
    if (daysUntilExpiry > 0 && daysUntilExpiry <= 7) {
      return 'warning';
    }
  }
  
  return 'good';
};

// Get CSS class for secret card based on health
const getSecretHealthClass = (secret: any): string => {
  const status = getSecretHealthStatus(secret);
  if (status === 'error') return 'secret-card-error';
  if (status === 'warning') return 'secret-card-warning';
  return '';
};

// Test a secret token
const testSecretToken = async (secretId: number) => {
  testingSecrets.value.add(secretId);
  secretHealthStatus.value.set(secretId, 'testing');
  
  try {
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore?.invoke) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    
    const result = await tauriCore.invoke<string>('test_github_token', { secretId });
    secretHealthStatus.value.set(secretId, 'good');
    alert(`✓ ${result}`);
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    secretHealthStatus.value.set(secretId, 'error');
    
    // Check if it's a 401/403 error
    if (errorMsg.includes('401') || errorMsg.includes('expired') || errorMsg.includes('invalid')) {
      alert(`❌ Token Test Failed: ${errorMsg}\n\nThis token appears to be invalid or expired. Please update it in the secret settings.`);
    } else if (errorMsg.includes('403') || errorMsg.includes('Forbidden')) {
      secretHealthStatus.value.set(secretId, 'warning');
      alert(`⚠️ Token Test Warning: ${errorMsg}\n\nThe token may be missing required permissions.`);
    } else {
      alert(`❌ Token Test Failed: ${errorMsg}`);
    }
  } finally {
    testingSecrets.value.delete(secretId);
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
  await buildSourceSecretMap();
});
</script>
