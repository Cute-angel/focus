<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const searchQuery = ref("");
const searchResults = ref<any[]>([]);
const selectedIndex = ref(0);

// 防抖搜索
let searchTimeout: NodeJS.Timeout;

async function performSearch(query: string) {
  if (!query.trim()) {
    searchResults.value = [];
    return;
  }

  try {
    const results = await invoke("get_file_finder_result", { textInput: query });
    searchResults.value = results;
    selectedIndex.value = 0;
  } catch (error) {
    console.error("搜索失败:", error);
    searchResults.value = [];
  }
}

function debounceSearch(query: string) {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    performSearch(query);
  }, 300);
}

// 关闭窗口
async function closeSpotlight() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const window = getCurrentWindow();
  await window.close();
}

// 选择结果
function selectResult(result: any) {
  console.log("选择结果:", result);
  // TODO: 实现打开文件功能
  closeSpotlight();
}

// 键盘导航
function handleKeyDown(event: KeyboardEvent) {
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      selectedIndex.value = Math.min(selectedIndex.value + 1, searchResults.value.length - 1);
      break;
    case 'ArrowUp':
      event.preventDefault();
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
      break;
    case 'Enter':
      event.preventDefault();
      if (searchResults.value[selectedIndex.value]) {
        selectResult(searchResults.value[selectedIndex.value]);
      }
      break;
    case 'Escape':
      event.preventDefault();
      closeSpotlight();
      break;
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown);
  document.querySelector('input')?.focus();
});
</script>

<template>
  <div class="spotlight-container" @click="closeSpotlight">
    <div class="spotlight-window" @click.stop>
      <!-- 搜索输入框 -->
      <div class="search-box">
        <div class="search-icon">🔍</div>
        <input v-model="searchQuery" @input="debounceSearch(searchQuery)" class="search-input" placeholder="搜索文件、文件夹..."
          autofocus />
        <div class="shortcuts">
          <kbd>↑↓</kbd> 导航
          <kbd>Enter</kbd> 选择
          <kbd>Esc</kbd> 关闭
        </div>
      </div>

      <!-- 搜索结果 -->
      <div v-if="searchResults.length > 0" class="results">
        <div v-for="(result, index) in searchResults" :key="index"
          :class="['result-item', { selected: index === selectedIndex }]" @click="selectResult(result)"
          @mouseenter="selectedIndex = index">
          <div class="result-icon">
            {{ result.is_folder ? '📁' : '📄' }}
          </div>
          <div class="result-info">
            <div class="result-name">{{ result.name }}</div>
            <div class="result-path">{{ result.path }}</div>
          </div>
        </div>
      </div>

      <!-- 无结果 -->
      <div v-else-if="searchQuery.trim()" class="no-results">
        <div class="no-results-icon">🔍</div>
        <div class="no-results-text">未找到匹配的文件</div>
      </div>

      <!-- 提示 -->
      <div v-else class="hints">
        <div class="hint-item">
          <span class="hint-icon">📄</span>
          <span class="hint-text">输入文件名进行搜索</span>
        </div>
        <div class="hint-item">
          <span class="hint-icon">⌨️</span>
          <span class="hint-text">使用键盘快速导航</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
:root {
  background-color: transparent;
}

.spotlight-container {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 20vh;
  z-index: 9999;
}

.spotlight-window {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  width: 90%;
  max-width: 600px;
  max-height: 70vh;
  overflow: hidden;
  animation: slideUp 0.3s ease;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.search-box {
  display: flex;
  align-items: center;
  padding: 1.2rem 1.5rem;
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  gap: 1rem;
}

.search-icon {
  font-size: 1.5rem;
  color: #667eea;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 1.1rem;
  color: #333;
}

.search-input::placeholder {
  color: rgba(0, 0, 0, 0.4);
}

.shortcuts {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: rgba(0, 0, 0, 0.5);
  flex-shrink: 0;
}

kbd {
  background: rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(0, 0, 0, 0.2);
  border-radius: 4px;
  padding: 2px 6px;
  font-family: monospace;
  font-size: 0.7rem;
  color: rgba(0, 0, 0, 0.7);
}

.results {
  max-height: 400px;
  overflow-y: auto;
}

.result-item {
  display: flex;
  align-items: center;
  padding: 1rem 1.5rem;
  cursor: pointer;
  transition: all 0.2s ease;
  gap: 1rem;
}

.result-item:hover,
.result-item.selected {
  background: rgba(102, 126, 234, 0.1);
}

.result-icon {
  font-size: 1.5rem;
  flex-shrink: 0;
}

.result-info {
  flex: 1;
  min-width: 0;
}

.result-name {
  font-weight: 600;
  color: #333;
  margin-bottom: 0.25rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-path {
  font-size: 0.85rem;
  color: rgba(0, 0, 0, 0.6);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.no-results {
  padding: 3rem 2rem;
  text-align: center;
  color: rgba(0, 0, 0, 0.6);
}

.no-results-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.no-results-text {
  font-size: 1.1rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.hints {
  padding: 2rem 1.5rem;
}

.hint-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 0;
  color: rgba(0, 0, 0, 0.7);
}

.hint-icon {
  font-size: 1.2rem;
  flex-shrink: 0;
}

.hint-text {
  font-size: 0.95rem;
}

/* 滚动条样式 */
.results::-webkit-scrollbar {
  width: 6px;
}

.results::-webkit-scrollbar-track {
  background: transparent;
}

.results::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 3px;
}

.results::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.3);
}
</style>