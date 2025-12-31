<template>
  <Transition name="fade">
    <div v-if="url" class="link-preview">
      <div class="link-preview-content">
        {{ url }}
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

const url = ref<string | null>(null);
let hideTimeout: ReturnType<typeof setTimeout> | null = null;
let currentLink: HTMLAnchorElement | null = null;

const showLink = (linkUrl: string) => {
  if (hideTimeout) {
    clearTimeout(hideTimeout);
    hideTimeout = null;
  }
  url.value = linkUrl;
};

const hideLink = () => {
  // Add a small delay before hiding to allow smooth transition
  hideTimeout = setTimeout(() => {
    url.value = null;
    currentLink = null;
  }, 150);
};

const handleMouseOver = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  // Check if the target is a link or inside a link
  const link = target.closest('a') as HTMLAnchorElement | null;
  
  if (link && link.href && link !== currentLink) {
    currentLink = link;
    showLink(link.href);
  } else if (!link && currentLink) {
    // Mouse moved away from the link
    hideLink();
  }
};

const handleMouseOut = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  const relatedTarget = e.relatedTarget as HTMLElement | null;
  
  // Check if we're leaving a link
  const link = target.closest('a') as HTMLAnchorElement | null;
  const relatedLink = relatedTarget?.closest('a') as HTMLAnchorElement | null;
  
  if (link && !relatedLink) {
    // We're leaving the link area
    hideLink();
  }
};

onMounted(() => {
  // Use event delegation on document body to catch all link hovers
  document.body.addEventListener('mouseover', handleMouseOver, true);
  document.body.addEventListener('mouseout', handleMouseOut, true);
});

onUnmounted(() => {
  document.body.removeEventListener('mouseover', handleMouseOver, true);
  document.body.removeEventListener('mouseout', handleMouseOut, true);
  if (hideTimeout) {
    clearTimeout(hideTimeout);
  }
});
</script>

<style scoped>
.link-preview {
  position: fixed;
  bottom: 20px;
  left: 20px;
  z-index: 1000;
  pointer-events: none;
}

.link-preview-content {
  background: rgba(0, 0, 0, 0.85);
  color: white;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  max-width: 600px;
  word-break: break-all;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(10px);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(10px);
}
</style>

