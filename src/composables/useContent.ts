import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Item } from '../types';

export interface ContentResolution {
  content: string;
  source: 'feed' | 'extracted';
  isFetching: boolean;
  hasError: boolean;
  errorMessage?: string;
}

export function useContent() {
  const articleViewMode = ref<string>('auto');
  const extractionEnabled = ref<boolean>(true);

  // Load preferences
  const loadPreferences = async () => {
    try {
      const mode = await invoke<string | null>('get_user_preference', { key: 'article_view_mode' });
      articleViewMode.value = mode || 'auto';
      
      const enabled = await invoke<string | null>('get_user_preference', { key: 'extraction_enabled' });
      extractionEnabled.value = enabled !== 'false';
    } catch (error) {
      console.error('Failed to load content preferences:', error);
    }
  };

  // Resolve which content to display based on preferences and item state
  const getDisplayContent = (item: Item | null): ContentResolution => {
    if (!item) {
      return {
        content: '',
        source: 'feed',
        isFetching: false,
        hasError: false,
      };
    }

    const status = item.content_status || '';
    const isFetching = status === 'fetching';
    const hasExtracted = status === 'extracted' && item.extracted_content_html;
    const hasFailed = status === 'failed';
    const completeness = item.content_completeness || 'unknown';

    // If extraction is disabled, always use feed content
    if (!extractionEnabled.value) {
      return {
        content: item.content_html || item.summary || '',
        source: 'feed',
        isFetching: false,
        hasError: false,
      };
    }

    // Handle different view modes
    if (articleViewMode.value === 'feed_only') {
      return {
        content: item.content_html || item.summary || '',
        source: 'feed',
        isFetching: false,
        hasError: false,
      };
    }

    if (articleViewMode.value === 'always_fetch') {
      if (hasExtracted) {
        return {
          content: item.extracted_content_html || '',
          source: 'extracted',
          isFetching: false,
          hasError: false,
        };
      }
      if (isFetching) {
        return {
          content: item.content_html || item.summary || '',
          source: 'feed',
          isFetching: true,
          hasError: false,
        };
      }
      if (hasFailed) {
        return {
          content: item.content_html || item.summary || '',
          source: 'feed',
          isFetching: false,
          hasError: true,
          errorMessage: item.extraction_failed_reason || 'Failed to extract content',
        };
      }
      // Trigger extraction if not attempted
      if (!status && item.url) {
        invoke('trigger_extraction', { itemId: item.id }).catch(console.error);
      }
      return {
        content: item.content_html || item.summary || '',
        source: 'feed',
        isFetching: false,
        hasError: false,
      };
    }

    // Auto mode (default)
    if (hasExtracted) {
      return {
        content: item.extracted_content_html || '',
        source: 'extracted',
        isFetching: false,
        hasError: false,
      };
    }

    if (isFetching) {
      return {
        content: item.content_html || item.summary || '',
        source: 'feed',
        isFetching: true,
        hasError: false,
      };
    }

    if (hasFailed) {
      return {
        content: item.content_html || item.summary || '',
        source: 'feed',
        isFetching: false,
        hasError: true,
        errorMessage: item.extraction_failed_reason || 'Failed to extract content',
      };
    }

    // If content is full, use feed content
    if (completeness === 'full') {
      return {
        content: item.content_html || item.summary || '',
        source: 'feed',
        isFetching: false,
        hasError: false,
      };
    }

    // If partial and not attempted, extraction should be triggered by background task
    // For now, show feed content
    return {
      content: item.content_html || item.summary || '',
      source: 'feed',
      isFetching: false,
      hasError: false,
    };
  };

  return {
    articleViewMode,
    extractionEnabled,
    loadPreferences,
    getDisplayContent,
  };
}
