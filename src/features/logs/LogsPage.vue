<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  DeleteOutlined,
  FolderOpenOutlined,
  PauseCircleOutlined,
  PlayCircleOutlined,
  ReloadOutlined,
  SearchOutlined,
  SwapOutlined,
} from '@ant-design/icons-vue'
import PageHeader from '@/shared/ui/PageHeader.vue'
import { api } from '@/shared/api/tauri'
import { commandErrorMessage } from '@/shared/api/commandError'
import { useAppStore } from '@/stores/appStore'
import type { AppLogEntry, AppLogLevel } from '@/shared/types'

type LogLevelFilter = 'ALL' | AppLogLevel
type SortDirection = 'asc' | 'desc'

const store = useAppStore()
const entries = ref<AppLogEntry[]>([])
const hiddenEntryIds = ref<Set<string>>(new Set())
const levelFilter = ref<LogLevelFilter>('ALL')
const searchText = ref('')
const caseSensitive = ref(false)
const wholeWord = ref(false)
const regexMode = ref(false)
const paused = ref(false)
const sortDirection = ref<SortDirection>('asc')
const loadingLogs = ref(false)
const errorMessage = ref('')
const logListRef = ref<HTMLElement | null>(null)

let refreshTimer: ReturnType<typeof window.setInterval> | undefined
let refreshRequestId = 0

const levelOptions: { value: LogLevelFilter; label: string }[] = [
  { value: 'ALL', label: 'ALL' },
  { value: 'ERROR', label: 'ERROR' },
  { value: 'WARN', label: 'WARNING' },
  { value: 'INFO', label: 'INFO' },
  { value: 'DEBUG', label: 'DEBUG' },
  { value: 'TRACE', label: 'TRACE' },
]

const visibleEntries = computed(() => entries.value.filter((entry) => !hiddenEntryIds.value.has(entry.id)))
const searchPattern = computed(() => buildSearchPattern())
const searchHasError = computed(() => Boolean(searchText.value.trim() && regexMode.value && !searchPattern.value))
const filteredEntries = computed(() => {
  return visibleEntries.value.filter((entry) => {
    if (levelFilter.value !== 'ALL' && normalizeLevel(entry.level) !== levelFilter.value) {
      return false
    }
    return matchesSearch(entry)
  })
})
const displayedEntries = computed(() => {
  return sortDirection.value === 'asc' ? filteredEntries.value : [...filteredEntries.value].reverse()
})
const hasActiveFilter = computed(() => {
  return levelFilter.value !== 'ALL' || Boolean(searchText.value.trim())
})
const summaryText = computed(() => `${filteredEntries.value.length} / ${visibleEntries.value.length} 行`)
const refreshStateText = computed(() => (paused.value ? '已暂停' : '实时'))
const emptyText = computed(() => {
  if (errorMessage.value) return errorMessage.value
  if (hasActiveFilter.value) return '没有匹配的日志'
  return '暂无日志'
})

async function refreshLogs(scrollToEnd = true) {
  if (loadingLogs.value) return
  const requestId = ++refreshRequestId
  loadingLogs.value = true
  try {
    const nextEntries = await api.readLogs(800)
    if (requestId !== refreshRequestId) return
    entries.value = nextEntries
    errorMessage.value = ''
    if (scrollToEnd) {
      await scrollToLiveEdge()
    }
  } catch (error) {
    if (requestId === refreshRequestId) {
      errorMessage.value = commandErrorMessage(error)
    }
  } finally {
    if (requestId === refreshRequestId) {
      loadingLogs.value = false
    }
  }
}

function togglePaused() {
  paused.value = !paused.value
  if (!paused.value) {
    void refreshLogs()
  }
}

function toggleSortDirection() {
  sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
}

function clearLogsView() {
  const nextHiddenIds = new Set(hiddenEntryIds.value)
  for (const entry of entries.value) {
    nextHiddenIds.add(entry.id)
  }
  hiddenEntryIds.value = nextHiddenIds
  void scrollToLiveEdge()
}

function buildSearchPattern() {
  const value = searchText.value.trim()
  if (!value) return null

  try {
    const pattern = regexMode.value ? value : escapeRegExp(value)
    const wrappedPattern = wholeWord.value ? `\\b(?:${pattern})\\b` : pattern
    return new RegExp(wrappedPattern, caseSensitive.value ? '' : 'i')
  } catch {
    return null
  }
}

function matchesSearch(entry: AppLogEntry) {
  const value = searchText.value.trim()
  if (!value) return true
  const pattern = searchPattern.value
  if (!pattern) return false
  return pattern.test(`${formatTimestamp(entry.timestamp)} ${entry.level} ${entry.target} ${entry.message} ${entry.raw}`)
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function normalizeLevel(level: string): AppLogLevel {
  const value = level.toUpperCase()
  if (value === 'TRACE' || value === 'DEBUG' || value === 'INFO' || value === 'ERROR') return value
  if (value === 'WARN' || value === 'WARNING') return 'WARN'
  return 'UNKNOWN'
}

function levelLabel(level: string) {
  const value = normalizeLevel(level)
  return value === 'WARN' ? 'WARNING' : value
}

function levelClass(level: string) {
  return `log-level-${normalizeLevel(level).toLowerCase()}`
}

function formatTimestamp(value: string) {
  if (!value) return '--'
  const parsed = new Date(value)
  if (Number.isNaN(parsed.getTime())) {
    const match = value.match(/(?:\d{4}-)?(\d{2}-\d{2})[T\s](\d{2}:\d{2}:\d{2})/)
    return match ? `${match[1]} ${match[2]}` : value
  }

  const pad = (item: number) => String(item).padStart(2, '0')
  return `${pad(parsed.getMonth() + 1)}-${pad(parsed.getDate())} ${pad(parsed.getHours())}:${pad(parsed.getMinutes())}:${pad(parsed.getSeconds())}`
}

async function scrollToLiveEdge() {
  await nextTick()
  const list = logListRef.value
  if (!list) return
  list.scrollTop = sortDirection.value === 'asc' ? list.scrollHeight : 0
}

watch([levelFilter, searchText, caseSensitive, wholeWord, regexMode, sortDirection], () => {
  void scrollToLiveEdge()
})

onMounted(() => {
  void refreshLogs()
  refreshTimer = window.setInterval(() => {
    if (!paused.value) {
      void refreshLogs()
    }
  }, 1500)
})

onBeforeUnmount(() => {
  if (refreshTimer !== undefined) {
    window.clearInterval(refreshTimer)
  }
})
</script>

<template>
  <div class="logs-page">
    <PageHeader title="日志" description="实时查看运行、健康检查与错误记录" class="logs-header">
      <template #actions>
        <a-tooltip :title="paused ? '继续刷新' : '暂停刷新'">
          <a-button class="log-live-button" :type="paused ? 'default' : 'primary'" ghost @click="togglePaused">
            <template #icon>
              <PlayCircleOutlined v-if="paused" />
              <PauseCircleOutlined v-else />
            </template>
            {{ refreshStateText }}
          </a-button>
        </a-tooltip>
        <a-tooltip :title="sortDirection === 'asc' ? '新日志在底部' : '新日志在顶部'">
          <a-button class="log-action-button" @click="toggleSortDirection">
            <template #icon><SwapOutlined /></template>
          </a-button>
        </a-tooltip>
        <a-tooltip title="重新读取">
          <a-button class="log-action-button" :loading="loadingLogs" @click="() => refreshLogs()">
            <template #icon><ReloadOutlined /></template>
          </a-button>
        </a-tooltip>
        <a-tooltip title="打开日志目录">
          <a-button class="log-action-button" :loading="store.loading" @click="store.openLogDir">
            <template #icon><FolderOpenOutlined /></template>
          </a-button>
        </a-tooltip>
        <a-popconfirm
          title="清空当前日志视图？"
          description="只隐藏当前显示的记录，不会删除日志文件。"
          ok-text="清空视图"
          cancel-text="取消"
          :ok-button-props="{ danger: true }"
          @confirm="clearLogsView"
        >
          <a-button danger :disabled="!visibleEntries.length">
            <template #icon><DeleteOutlined /></template>
            清空视图
          </a-button>
        </a-popconfirm>
      </template>
    </PageHeader>

    <section class="log-console" aria-label="日志控制台">
      <div class="log-toolbar">
        <a-select v-model:value="levelFilter" class="log-level-select" size="large">
          <a-select-option v-for="option in levelOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </a-select-option>
        </a-select>

        <a-input
          v-model:value="searchText"
          class="log-search-input"
          :status="searchHasError ? 'error' : undefined"
          size="large"
          allow-clear
          placeholder="过滤条件"
        >
          <template #prefix><SearchOutlined /></template>
          <template #suffix>
            <span class="log-search-modes">
              <a-tooltip title="区分大小写">
                <button
                  type="button"
                  class="log-mode-button"
                  :class="{ 'log-mode-button-active': caseSensitive }"
                  aria-label="区分大小写"
                  @click.stop="caseSensitive = !caseSensitive"
                >
                  Aa
                </button>
              </a-tooltip>
              <a-tooltip title="整词匹配">
                <button
                  type="button"
                  class="log-mode-button"
                  :class="{ 'log-mode-button-active': wholeWord }"
                  aria-label="整词匹配"
                  @click.stop="wholeWord = !wholeWord"
                >
                  ab
                </button>
              </a-tooltip>
              <a-tooltip title="正则表达式">
                <button
                  type="button"
                  class="log-mode-button"
                  :class="{ 'log-mode-button-active': regexMode }"
                  aria-label="正则表达式"
                  @click.stop="regexMode = !regexMode"
                >
                  .*
                </button>
              </a-tooltip>
            </span>
          </template>
        </a-input>

        <span class="log-summary">{{ summaryText }}</span>
      </div>

      <div ref="logListRef" class="app-scrollbar log-list">
        <div v-if="!displayedEntries.length" class="log-empty" :class="{ 'log-empty-error': errorMessage }">
          {{ emptyText }}
        </div>
        <template v-else>
          <div
            v-for="entry in displayedEntries"
            :key="entry.id"
            class="log-row"
          >
            <span class="log-time">{{ formatTimestamp(entry.timestamp) }}</span>
            <span class="log-level" :class="levelClass(entry.level)">{{ levelLabel(entry.level) }}</span>
            <span class="log-message">
              <span v-if="entry.target" class="log-target">[{{ entry.target }}]</span>
              {{ entry.message || entry.raw }}
            </span>
          </div>
        </template>
      </div>
    </section>
  </div>
</template>

<style scoped>
.logs-page {
  display: flex;
  height: 100%;
  min-height: 0;
  flex-direction: column;
}

.logs-header {
  flex: 0 0 auto;
}

.log-live-button {
  min-width: 84px;
}

.log-action-button {
  width: 36px;
  padding-inline: 0;
}

.log-console {
  display: flex;
  min-height: 360px;
  flex: 1 1 auto;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--line-soft);
  border-radius: 10px;
  background: var(--panel-bg);
  box-shadow: var(--panel-shadow);
}

.log-toolbar {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 12px;
  border-bottom: 1px solid var(--line-soft);
  background: var(--panel-bg);
  padding: 14px;
}

.log-level-select {
  width: 158px;
  flex: 0 0 158px;
}

.log-search-input {
  min-width: 220px;
  flex: 1 1 auto;
}

.log-search-modes {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding-left: 6px;
}

.log-mode-button {
  display: inline-flex;
  min-width: 25px;
  height: 24px;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 4px;
  color: #9297a0;
  background: transparent;
  cursor: default;
  font-size: 12px;
  font-weight: 600;
  line-height: 1;
  padding: 0 4px;
}

.log-mode-button:hover,
.log-mode-button-active {
  color: #1677ff;
  background: rgba(22, 119, 255, 0.1);
}

.log-mode-button-active {
  box-shadow: inset 0 0 0 1px rgba(37, 99, 235, 0.3);
}

.log-summary {
  min-width: 86px;
  color: var(--text-muted);
  font-size: 12px;
  text-align: right;
}

.log-list {
  min-height: 0;
  flex: 1 1 auto;
  overflow: auto;
  background: #f2f3f5;
}

.log-row {
  display: grid;
  min-height: 38px;
  grid-template-columns: 150px 88px minmax(0, 1fr);
  align-items: start;
  column-gap: 8px;
  border-bottom: 1px solid #dfe2e7;
  padding: 8px 16px;
  color: var(--text-primary);
  font-size: 14px;
  font-family: "JetBrains Mono", "Cascadia Mono", Consolas, monospace;
  line-height: 1.45;
}

.log-time {
  color: #858b95;
  white-space: nowrap;
}

.log-level {
  font-weight: 700;
  white-space: nowrap;
}

.log-level-trace {
  color: #64748b;
}

.log-level-debug {
  color: #0f766e;
}

.log-level-info {
  color: #1677ff;
}

.log-level-warn {
  color: #f59e0b;
}

.log-level-error {
  color: #ef4444;
}

.log-level-unknown {
  color: var(--text-muted);
}

.log-message {
  min-width: 0;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

.log-target {
  margin-right: 6px;
  color: #6b7280;
}

.log-empty {
  display: flex;
  height: 100%;
  min-height: 240px;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 14px;
}

.log-empty-error {
  color: #ef4444;
}

:global(:root[data-theme="dark"]) .log-console {
  box-shadow: none;
}

:global(:root[data-theme="dark"]) .log-list {
  background: #151517;
}

:global(:root[data-theme="dark"]) .log-row {
  border-color: #26262b;
}

:global(:root[data-theme="dark"]) .log-time,
:global(:root[data-theme="dark"]) .log-target {
  color: #9ca3af;
}

@media (max-width: 760px) {
  .log-toolbar {
    flex-wrap: wrap;
  }

  .log-level-select {
    width: 132px;
    flex-basis: 132px;
  }

  .log-search-input {
    min-width: 100%;
    order: 3;
  }

  .log-summary {
    margin-left: auto;
  }

  .log-row {
    grid-template-columns: minmax(0, 1fr);
    row-gap: 2px;
  }
}
</style>
