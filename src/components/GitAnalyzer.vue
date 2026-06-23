<template>
  <div class="git-analyzer">
    <header class="page-header">
      <h1 class="page-title">
        <el-icon><Files /></el-icon>
        Git 导出器
      </h1>
      <p class="page-subtitle">按作者与日期范围导出 Git 提交记录</p>
    </header>

    <main class="page-main">
      <!-- 配置导入 -->
      <el-card class="section-card" shadow="never">
        <template #header>
          <div class="section-header">
            <div class="section-header__title">
              <el-icon class="section-header__icon"><Collection /></el-icon>
              <span>上次配置</span>
            </div>
          </div>
        </template>
        <div class="toolbar-row">
          <el-select
            v-model="selectedConfigIndex"
            placeholder="选择上次配置"
            class="toolbar-grow"
            fit-input-width
            popper-class="app-select-dropdown"
            clearable
            @change="applyConfig"
          >
            <el-option
              v-for="(cfg, idx) in savedConfigs"
              :key="idx"
              :label="cfg.label"
              :value="idx"
            />
          </el-select>
          <el-button class="toolbar-btn" type="primary" plain @click="saveCurrentAsConfig">
            保存配置
          </el-button>
          <el-button
            class="toolbar-btn"
            :icon="EditPen"
            :disabled="selectedConfigIndex === null"
            @click="renameSelectedConfig"
          >
            重命名
          </el-button>
        </div>
      </el-card>

      <!-- 项目路径 -->
      <el-card class="section-card" shadow="never">
        <template #header>
          <div class="section-header">
            <div class="section-header__title">
              <el-icon class="section-header__icon"><FolderOpened /></el-icon>
              <span>项目路径</span>
            </div>
          </div>
        </template>
        <div class="toolbar-row">
          <el-input
            v-model="projectPath"
            placeholder="输入或选择本地 Git 项目目录"
            clearable
            class="toolbar-grow"
          >
            <template #prefix>
              <el-icon><Monitor /></el-icon>
            </template>
          </el-input>
          <el-button class="toolbar-btn" type="primary" @click="browsePath" :icon="Search">
            选择目录
          </el-button>
          <el-button
            class="toolbar-btn"
            type="success"
            @click="loadAuthors"
            :loading="loading"
            :disabled="!projectPath"
            :icon="Refresh"
          >
            {{ loading ? '加载中' : '加载作者' }}
          </el-button>
        </div>
        <div class="toolbar-row">
          <el-select
            v-model="selectedSavedProjectPath"
            placeholder="选择已保存的项目路径"
            class="toolbar-grow"
            fit-input-width
            popper-class="app-select-dropdown project-path-select"
            clearable
            @change="applySavedProjectPath"
          >
            <el-option
              v-for="path in savedProjectPaths"
              :key="path"
              :label="getProjectPathLabel(path)"
              :value="path"
            >
              <div class="project-path-row">
                <span class="project-path-text" :title="path">{{ getProjectPathLabel(path) }}</span>
                <el-icon class="project-path-delete" @click.stop="removeSavedProjectPath(path)">
                  <Close />
                </el-icon>
              </div>
            </el-option>
            <template v-if="savedProjectPaths.length" #footer>
              <div class="project-path-footer">
                <el-button text type="danger" size="small" @click="clearAllProjectPaths">
                  清空全部
                </el-button>
              </div>
            </template>
          </el-select>
          <el-button
            class="toolbar-btn"
            type="primary"
            plain
            :disabled="!projectPath.trim() || isProjectPathSaved(projectPath)"
            @click="saveCurrentProjectPath"
          >
            保存路径
          </el-button>
        </div>
      </el-card>

      <!-- 作者选择 -->
      <el-card v-if="authors.length > 0" class="section-card" shadow="never">
        <template #header>
          <div class="section-header">
            <div class="section-header__title">
              <el-icon class="section-header__icon"><User /></el-icon>
              <span>选择作者</span>
              <el-tag type="primary" size="small" effect="plain">
                {{ selectedAuthors.length ? `已选 ${selectedAuthors.length}/${authors.length}` : '全部作者' }}
              </el-tag>
            </div>
            <div class="section-header__actions">
              <el-button class="toolbar-btn" plain @click="selectAllAuthors">全选</el-button>
              <el-button class="toolbar-btn" plain @click="selectNoneAuthors">清空</el-button>
            </div>
          </div>
        </template>

        <el-select
          v-model="selectedAuthorUids"
          multiple
          filterable
          collapse-tags
          collapse-tags-tooltip
          :max-collapse-tags="3"
          fit-input-width
          popper-class="app-select-dropdown"
          reserve-keyword
          filter-method="filterAuthors"
          remote
          :remote-method="filterAuthors"
          placeholder="未选择 = 全部作者；输入名称搜索并添加"
          class="w-full"
          clearable
          @change="handleAuthorChange"
        >
          <el-option
            v-for="author in filteredAuthors"
            :key="author._uid"
            :label="`${author.name}(${author.email})`"
            :value="author._uid"
          >
            <div class="author-option">
              <div class="author-option-main">
                <span class="author-option-check">{{ isSelectedByUid(author._uid) ? '✅' : '⬜' }}</span>
                <span class="author-option-text">{{ author.name}}({{ author.email }})</span>
              </div>
              <el-tag size="small" class="author-option-tag">{{ author.count }} commits</el-tag>
            </div>
          </el-option>
        </el-select>

        <p v-if="!selectedAuthors.length" class="form-hint">
          当前未选择特定作者，将包含全部 {{ authors.length }} 位作者的提交
        </p>

        <div v-if="selectedAuthors.length" class="scroll-panel author-tags-scroll">
          <div class="scroll-panel__label">已选 {{ selectedAuthors.length }} 位作者</div>
          <div class="tag-list">
            <el-tag
              v-for="a in selectedAuthors"
              :key="a._uid"
              closable
              type="primary"
              size="small"
              class="tag-list__item"
              @close="removeAuthor(a)"
            >
              {{ a.name}}({{ a.email }})
            </el-tag>
          </div>
        </div>
      </el-card>

      <!-- 日期范围 -->
      <el-card class="section-card" shadow="never">
        <template #header>
          <div class="section-header">
            <div class="section-header__title">
              <el-icon class="section-header__icon"><Calendar /></el-icon>
              <span>日期范围</span>
            </div>
          </div>
        </template>
        <el-radio-group v-model="dateMode" class="mode-switch">
          <el-radio-button value="single">单个日期</el-radio-button>
          <el-radio-button value="range">时间段</el-radio-button>
        </el-radio-group>

        <div v-if="dateMode === 'single'" class="toolbar-row">
          <el-date-picker
            v-model="singleDate"
            type="date"
            placeholder="选择日期"
            format="YYYY-MM-DD"
            value-format="YYYY-MM-DD"
            class="toolbar-grow"
            clearable
          />
        </div>

        <div v-else class="date-range-block">
          <div v-for="(range, index) in dateRanges" :key="index" class="toolbar-row">
            <el-date-picker
              v-model="range.start"
              type="date"
              placeholder="开始日期"
              format="YYYY-MM-DD"
              value-format="YYYY-MM-DD"
              class="toolbar-grow"
              clearable
            />
            <span class="range-separator">至</span>
            <el-date-picker
              v-model="range.end"
              type="date"
              placeholder="结束日期"
              format="YYYY-MM-DD"
              value-format="YYYY-MM-DD"
              class="toolbar-grow"
              clearable
            />
            <el-popconfirm
              v-if="dateRanges.length > 1"
              title="确认删除此时间段？"
              confirm-button-text="删除"
              cancel-button-text="取消"
              @confirm="removeRange(index)"
            >
              <template #reference>
                <el-button class="toolbar-btn toolbar-btn--icon" type="danger" :icon="Delete" circle />
              </template>
            </el-popconfirm>
          </div>
          <el-button class="toolbar-btn" type="primary" plain :icon="Plus" @click="addRange">
            添加时间段
          </el-button>
        </div>
      </el-card>

      <!-- 导出设置 -->
      <el-card class="section-card" shadow="never">
        <template #header>
          <div class="section-header">
            <div class="section-header__title">
              <el-icon class="section-header__icon"><Setting /></el-icon>
              <span>导出设置</span>
            </div>
          </div>
        </template>
        <div class="export-grid">
          <div class="export-col">
            <el-form label-position="top" class="uniform-form">
              <el-form-item label="导出格式">
                <el-select v-model="exportFormat" class="w-full" fit-input-width popper-class="app-select-dropdown">
                  <el-option label="TXT 文本" value="txt" />
                  <el-option label="CSV 表格" value="csv" />
                  <el-option label="JSON 数据" value="json" />
                  <el-option label="Markdown" value="md" />
                </el-select>
              </el-form-item>
              <el-form-item label="导出模板">
                <el-select v-model="exportTemplate" class="w-full" fit-input-width popper-class="app-select-dropdown">
                  <el-option label="完整版（含作者信息）" value="full" />
                  <el-option label="精简版（仅提交记录）" value="compact" />
                </el-select>
              </el-form-item>
              <el-form-item label="增量导出">
                <el-switch v-model="incrementalExport" />
              </el-form-item>
            </el-form>
          </div>
          <div class="export-col">
            <el-form label-position="top" class="uniform-form">
              <el-form-item label="文件名称">
                <el-input
                  v-model="exportFilename"
                  placeholder="预览数据后自动生成，可修改"
                  :disabled="!previewCommits.length"
                  clearable
                  @input="filenameEdited = true"
                >
                  <template #append>
                    <el-button :disabled="!previewCommits.length" @click="resetExportFilename">
                      重置
                    </el-button>
                  </template>
                </el-input>
                <p v-if="!previewCommits.length" class="form-hint">
                  请先预览数据，系统将自动生成文件名
                </p>
                <p v-else-if="exportFullPathPreview" class="form-hint">
                  保存路径：{{ exportFullPathPreview }}
                </p>
              </el-form-item>
              <el-form-item label="导出目录">
                <div class="toolbar-row toolbar-row--compact">
                  <el-input
                    v-model="exportDir"
                    placeholder="选择导出目录"
                    readonly
                    clearable
                    class="toolbar-grow"
                  >
                    <template #prefix>
                      <el-icon><Folder /></el-icon>
                    </template>
                  </el-input>
                  <el-button class="toolbar-btn" @click="browseExportDir" :icon="Search">选择</el-button>
                </div>
              </el-form-item>
            </el-form>
          </div>
        </div>
      </el-card>

      <!-- 操作按钮 -->
      <el-card class="section-card action-card" shadow="never">
        <div class="action-bar">
          <el-button
            class="action-btn"
            type="warning"
            :disabled="!hasValidDateFilter() && !incrementalExport"
            :loading="previewLoading"
            :icon="Document"
            @click="previewData"
          >
            {{ previewLoading ? '加载中' : '预览数据' }}
          </el-button>
          <el-button
            class="action-btn"
            type="primary"
            :disabled="(!hasValidDateFilter() && !incrementalExport) || !previewCommits.length || !exportFilename.trim()"
            :icon="Download"
            @click="exportData"
          >
            导出为 {{ formatLabel }}
          </el-button>
        </div>
      </el-card>

      <!-- 预览 -->
      <el-card v-if="previewCommits.length > 0" class="section-card" shadow="never">
        <template #header>
          <div class="section-header">
            <div class="section-header__title">
              <el-icon class="section-header__icon"><Document /></el-icon>
              <span>提交预览</span>
              <el-tag type="success" size="small" effect="plain">{{ previewCommits.length }} 条</el-tag>
            </div>
            <div class="section-header__actions">
              <el-button class="toolbar-btn" plain @click="previewCommits = []">收起</el-button>
            </div>
          </div>
        </template>
        <div class="scroll-panel preview-panel">
          <pre class="preview-text">{{ previewText }}</pre>
        </div>
      </el-card>

      <!-- 导出历史 -->
      <el-card v-if="exportHistory.length > 0" class="section-card" shadow="never">
        <template #header>
          <div class="section-header">
            <div class="section-header__title">
              <el-icon class="section-header__icon"><Clock /></el-icon>
              <span>导出历史</span>
            </div>
            <div class="section-header__actions">
              <el-button class="toolbar-btn" type="danger" plain @click="clearHistory">清空</el-button>
            </div>
          </div>
        </template>
        <el-timeline class="history-timeline">
          <el-timeline-item
            v-for="(record, idx) in exportHistory"
            :key="idx"
            :timestamp="record.timestamp"
            placement="top"
            type="primary"
          >
            <div class="history-item">
              <div class="history-item__head">
                <el-tag size="small" :type="formatTagType(record.format)">
                  {{ record.format.toUpperCase() }}
                </el-tag>
                <span class="history-item__name">{{ record.filename }}</span>
                <el-tag v-if="record.incremental" size="small" type="warning" effect="plain">增量</el-tag>
              </div>
              <p class="form-hint">
                {{ record.author_count }} 作者 · {{ record.commit_count }} 提交 · {{ record.project_path }}
              </p>
              <p v-if="record.file_path" class="history-item__path">
                <span
                  class="history-item__link"
                  :title="record.file_path"
                  @click="openRecordDirectory(record)"
                >
                  <el-icon><Folder /></el-icon>
                  {{ getDirFromFilePath(record.file_path) }}
                </span>
              </p>
            </div>
          </el-timeline-item>
        </el-timeline>
      </el-card>
    </main>

    <!-- 导出成功弹窗 -->
    <el-dialog
      v-model="showSuccessDialog"
      title="导出成功"
      width="420px"
      :icon="CircleCheckFilled"
    >
      <div class="dialog-body">
        <p><strong>文件：</strong>{{ successFilePath }}</p>
      </div>
      <template #footer>
        <div class="dialog-footer">
          <el-button type="primary" @click="handleOpenDirectory">
            <el-icon class="mr-1"><Folder /></el-icon>
            打开导出目录
          </el-button>
          <el-button @click="showSuccessDialog = false">关闭</el-button>
        </div>
      </template>
    </el-dialog>

    <footer class="page-footer">v{{ appVersion }}</footer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  Files, FolderOpened, Monitor, Search, Refresh,
  User, Calendar, Setting, Document, Download,
  Delete, Plus, Clock, Folder,
  CircleCheckFilled, Collection, EditPen, Close,
} from '@element-plus/icons-vue';

interface Author {
  name: string;
  email: string;
  count: number;
  _uid: string;
}

interface DateRange {
  start: string;
  end: string;
}

interface GitCommit {
  hash: string;
  author_name: string;
  author_email: string;
  date: string;
  message: string;
}

interface ExportRecord {
  timestamp: string;
  project_path: string;
  filename: string;
  format: string;
  author_count: number;
  commit_count: number;
  incremental: boolean;
  file_path?: string;
}

const projectPath = ref('');
const savedProjectPaths = ref<string[]>([]);
const selectedSavedProjectPath = ref('');
const authors = ref<Author[]>([]);
const selectedAuthors = ref<Author[]>([]);
const selectedAuthorUids = ref<string[]>([]);
const loading = ref(false);
const previewLoading = ref(false);
const dateMode = ref<'single' | 'range'>('single');
const singleDate = ref('');
const dateRanges = ref<DateRange[]>([{ start: '', end: '' }]);
const previewCommits = ref<GitCommit[]>([]);

const exportFormat = ref<'txt' | 'csv' | 'json' | 'md'>('txt');
const exportTemplate = ref<'full' | 'compact'>('full');
const incrementalExport = ref(false);
const exportHistory = ref<ExportRecord[]>([]);
const exportDir = ref('');
const exportFilename = ref('');
const filenameEdited = ref(false);

const appVersion = ref('');

const filteredAuthors = ref<Author[]>([]);

const showSuccessDialog = ref(false);
const successFilePath = ref('');
const successDirPath = ref('');

// 配置管理
interface SavedConfig {
  label: string;
  project_path: string;
  selected_author_uids: string[];
  date_mode: string;
  single_date: string;
  date_ranges: DateRange[];
  export_format: string;
  export_template: string;
  incremental_export: boolean;
  export_dir: string;
}
const savedConfigs = ref<SavedConfig[]>([]);
const selectedConfigIndex = ref<number | null>(null);

const formatLabel = computed(() => {
  const map: Record<string, string> = { txt: 'TXT', csv: 'CSV', json: 'JSON', md: 'Markdown' };
  return map[exportFormat.value] || exportFormat.value.toUpperCase();
});

const exportFullPathPreview = computed(() => {
  const filename = sanitizeFilename(exportFilename.value);
  if (!filename) return '';
  if (exportDir.value) {
    const dir = exportDir.value.replace(/\/+$/, '');
    return `${dir}/${filename}`;
  }
  return filename;
});

const formatTagType = (fmt: string) => {
  const map: Record<string, 'success' | 'primary' | 'warning' | 'info'> = {
    txt: 'info', csv: 'success', json: 'primary', md: 'warning'
  };
  return map[fmt] || 'info';
};

onMounted(async () => {
  try {
    const history: ExportRecord[] = await invoke('get_export_history');
    exportHistory.value = history;
  } catch (e) {
    console.error('加载历史记录失败:', e);
  }
  // 加载版本号
  try {
    appVersion.value = await invoke('get_app_version');
  } catch (e) {
    console.error('加载版本号失败:', e);
    appVersion.value = '0.0.0';
  }
  // 加载保存的配置列表
  loadSavedConfigs();
  loadSavedProjectPaths();
});

watch(exportFormat, () => {
  if (!exportFilename.value) return;
  if (!filenameEdited.value) {
    resetExportFilename();
    return;
  }
  const base = exportFilename.value.replace(/\.[^.]+$/, '');
  exportFilename.value = `${base}.${exportFormat.value}`;
});

watch(() => previewCommits.value.length, (len) => {
  if (len === 0) {
    exportFilename.value = '';
    filenameEdited.value = false;
  }
});

// 加载配置列表
function loadSavedConfigs() {
  try {
    const raw = localStorage.getItem('git-analyzer-configs');
    if (raw) {
      savedConfigs.value = JSON.parse(raw);
    }
  } catch (e) {
    console.error('加载配置失败:', e);
  }
}

// 保存配置列表
function saveConfigsList() {
  localStorage.setItem('git-analyzer-configs', JSON.stringify(savedConfigs.value));
}

const PROJECT_PATHS_KEY = 'git-analyzer-project-paths';
const MAX_SAVED_PROJECT_PATHS = 20;

function loadSavedProjectPaths() {
  try {
    const raw = localStorage.getItem(PROJECT_PATHS_KEY);
    if (raw) {
      savedProjectPaths.value = JSON.parse(raw);
    }
  } catch (e) {
    console.error('加载项目路径失败:', e);
  }
}

function saveProjectPathsList() {
  localStorage.setItem(PROJECT_PATHS_KEY, JSON.stringify(savedProjectPaths.value));
}

function normalizeProjectPath(path: string): string {
  return path.trim().replace(/\/+$/, '');
}

function getProjectPathLabel(path: string): string {
  const normalized = normalizeProjectPath(path);
  const name = normalized.split('/').filter(Boolean).pop();
  return name || normalized;
}

function isProjectPathSaved(path: string): boolean {
  const normalized = normalizeProjectPath(path);
  return savedProjectPaths.value.some(p => normalizeProjectPath(p) === normalized);
}

function saveCurrentProjectPath() {
  const path = normalizeProjectPath(projectPath.value);
  if (!path) return;
  if (isProjectPathSaved(path)) {
    ElMessage.warning('该路径已保存');
    return;
  }
  savedProjectPaths.value.unshift(path);
  if (savedProjectPaths.value.length > MAX_SAVED_PROJECT_PATHS) {
    savedProjectPaths.value = savedProjectPaths.value.slice(0, MAX_SAVED_PROJECT_PATHS);
  }
  projectPath.value = path;
  selectedSavedProjectPath.value = path;
  saveProjectPathsList();
  ElMessage.success('项目路径已保存');
}

function applySavedProjectPath(path: string | null) {
  if (!path) return;
  projectPath.value = path;
  selectedSavedProjectPath.value = path;
}

function removeSavedProjectPath(path: string) {
  savedProjectPaths.value = savedProjectPaths.value.filter(p => p !== path);
  if (selectedSavedProjectPath.value === path) {
    selectedSavedProjectPath.value = '';
  }
  saveProjectPathsList();
}

async function clearAllProjectPaths() {
  if (savedProjectPaths.value.length === 0) return;
  try {
    await ElMessageBox.confirm(
      `确定清空全部 ${savedProjectPaths.value.length} 个已保存的项目路径？`,
      '清空全部',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
    savedProjectPaths.value = [];
    selectedSavedProjectPath.value = '';
    saveProjectPathsList();
    ElMessage.success('已清空全部项目路径');
  } catch {
    // 用户取消
  }
}

watch(projectPath, (path) => {
  const normalized = normalizeProjectPath(path);
  if (!normalized) {
    selectedSavedProjectPath.value = '';
    return;
  }
  const matched = savedProjectPaths.value.find(p => normalizeProjectPath(p) === normalized);
  selectedSavedProjectPath.value = matched || '';
});

// 保存当前为配置
async function saveCurrentAsConfig() {
  const defaultLabel = `${projectPath.value.split('/').filter(Boolean).pop() || '项目'} - ${new Date().toLocaleDateString('zh-CN')}`;
  let label: string;
  try {
    const { value } = await ElMessageBox.prompt('请输入配置名称', '保存配置', {
      confirmButtonText: '保存',
      cancelButtonText: '取消',
      inputValue: defaultLabel,
      inputValidator: (val) => (val?.trim() ? true : '名称不能为空'),
    });
    label = value.trim();
  } catch {
    return;
  }

  const config: SavedConfig = {
    label,
    project_path: projectPath.value,
    selected_author_uids: [...selectedAuthorUids.value],
    date_mode: dateMode.value,
    single_date: singleDate.value,
    date_ranges: [...dateRanges.value],
    export_format: exportFormat.value,
    export_template: exportTemplate.value,
    incremental_export: incrementalExport.value,
    export_dir: exportDir.value,
  };
  savedConfigs.value.unshift(config);
  if (savedConfigs.value.length > 20) {
    savedConfigs.value = savedConfigs.value.slice(0, 20);
  }
  selectedConfigIndex.value = 0;
  saveConfigsList();
  ElMessage.success('配置已保存');
}

async function renameSelectedConfig() {
  if (selectedConfigIndex.value === null) return;
  const idx = selectedConfigIndex.value;
  const cfg = savedConfigs.value[idx];
  try {
    const { value } = await ElMessageBox.prompt('请输入新的配置名称', '重命名配置', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      inputValue: cfg.label,
      inputValidator: (val) => (val?.trim() ? true : '名称不能为空'),
    });
    savedConfigs.value[idx].label = value.trim();
    saveConfigsList();
    ElMessage.success('配置已重命名');
  } catch {
    // 用户取消
  }
}

// 应用配置
function applyConfig(idx: number | null) {
  if (idx === null || idx >= savedConfigs.value.length) return;
  const cfg = savedConfigs.value[idx];
  projectPath.value = cfg.project_path;
  selectedAuthorUids.value = [...cfg.selected_author_uids];
  selectedAuthors.value = cfg.selected_author_uids
    .map(uid => authors.value.find(a => a._uid === uid))
    .filter((a): a is Author => !!a);
  dateMode.value = cfg.date_mode as 'single' | 'range';
  singleDate.value = cfg.single_date;
  dateRanges.value = [...cfg.date_ranges];
  exportFormat.value = cfg.export_format as 'txt' | 'csv' | 'json' | 'md';
  exportTemplate.value = cfg.export_template as 'full' | 'compact';
  incrementalExport.value = cfg.incremental_export;
  exportDir.value = cfg.export_dir;
  filteredAuthors.value = [...authors.value];
}

// 搜索过滤作者
function filterAuthors(query: string) {
  if (!query) {
    filteredAuthors.value = authors.value;
  } else {
    const q = query.toLowerCase();
    filteredAuthors.value = authors.value.filter(a =>
      a.name.toLowerCase().includes(q) || a.email.toLowerCase().includes(q)
    );
  }
}

function isSelectedByUid(uid: string): boolean {
  return selectedAuthorUids.value.includes(uid);
}

// 当 el-select 的 value 变化时，同步 selectedAuthors
function handleAuthorChange(uids: string[]) {
  selectedAuthorUids.value = uids;
  selectedAuthors.value = uids.map(uid => authors.value.find(a => a._uid === uid)).filter((a): a is Author => !!a);
}

function removeAuthor(author: Author) {
  const idx = selectedAuthorUids.value.indexOf(author._uid);
  if (idx >= 0) {
    selectedAuthorUids.value.splice(idx, 1);
    selectedAuthors.value.splice(idx, 1);
  }
}

function selectAllAuthors() {
  selectedAuthorUids.value = authors.value.map(a => a._uid);
  selectedAuthors.value = [...authors.value];
}

function selectNoneAuthors() {
  selectedAuthorUids.value = [];
  selectedAuthors.value = [];
}

function hasValidDateFilter(): boolean {
  if (dateMode.value === 'single') {
    return !!singleDate.value;
  }
  return dateRanges.value.some(r => r.start && r.end);
}

function getAuthorEmailsForQuery(): string[] {
  return selectedAuthors.value.length > 0
    ? selectedAuthors.value.map(a => a.email)
    : [];
}

function getEffectiveAuthorCount(): number {
  return selectedAuthors.value.length > 0 ? selectedAuthors.value.length : authors.value.length;
}

const browsePath = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择项目目录'
    });
    if (selected) {
      projectPath.value = selected as string;
    }
  } catch (error) {
    console.error('选择目录失败:', error);
    const path = prompt('输入项目路径:');
    if (path) projectPath.value = path;
  }
};

const browseExportDir = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择导出目录'
    });
    if (selected) {
      exportDir.value = selected as string;
    }
  } catch (error) {
    console.error('选择导出目录失败:', error);
    const dir = prompt('输入导出目录路径:');
    if (dir) exportDir.value = dir;
  }
};

const loadAuthors = async () => {
  if (!projectPath.value) return;
  loading.value = true;
  try {
    const result: Author[] = await invoke('get_git_authors', { projectPath: projectPath.value });
    authors.value = result.map(a => ({
      ...a,
      _uid: `${a.name}|||${a.email}`
    }));
    filteredAuthors.value = [...authors.value];
  } catch (error) {
    console.error('加载作者失败:', error);
    alert(`加载作者失败: ${error}`);
  } finally {
    loading.value = false;
  }
};

const addRange = () => {
  dateRanges.value.push({ start: '', end: '' });
};

const removeRange = (index: number) => {
  if (dateRanges.value.length > 1) {
    dateRanges.value.splice(index, 1);
  }
};

const previewText = computed(() => {
  const lines: string[] = [];
  previewCommits.value.forEach(commit => {
    const date = new Date(commit.date);
    const dateStr = `${date.getFullYear()}-${String(date.getMonth()+1).padStart(2,'0')}-${String(date.getDate()).padStart(2,'0')} ${String(date.getHours()).padStart(2,'0')}:${String(date.getMinutes()).padStart(2,'0')}:${String(date.getSeconds()).padStart(2,'0')}`;
    const msg = commit.message.replace(/\r?\n/g, ' ');
    lines.push(`${commit.hash} - ${dateStr} - ${commit.author_name}(${commit.author_email}) : ${msg}`);
  });
  return lines.join('\n');
});

function getFilename(baseName: string): string {
  const now = new Date();
  const ts = `${now.getFullYear()}${String(now.getMonth()+1).padStart(2,'0')}${String(now.getDate()).padStart(2,'0')}_${String(now.getHours()).padStart(2,'0')}${String(now.getMinutes()).padStart(2,'0')}`;
  return `git-export-${baseName}-${ts}.${exportFormat.value}`;
}

function sanitizeFilename(name: string): string {
  const cleaned = name.replace(/[/\\:*?"<>|]/g, '_').trim();
  if (!cleaned) return '';
  return cleaned.includes('.') ? cleaned : `${cleaned}.${exportFormat.value}`;
}

function resetExportFilename() {
  filenameEdited.value = false;
  const baseName = projectPath.value.split('/').filter(Boolean).pop() || 'repo';
  exportFilename.value = getFilename(baseName);
}

function fmtDate(iso: string): string {
  const d = new Date(iso);
  return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')} ${String(d.getHours()).padStart(2,'0')}:${String(d.getMinutes()).padStart(2,'0')}:${String(d.getSeconds()).padStart(2,'0')}`;
}

function buildReportContent(commits: GitCommit[], format: string, template: string): string {
  const now = new Date().toISOString();

  if (template === 'compact') {
    if (format === 'txt') {
      const lines = commits.map(c => {
        const ds = fmtDate(c.date);
        return `${c.hash} - ${ds} - ${c.author_name}(${c.author_email}) : ${c.message.replace(/\r?\n/g, ' ')}`;
      });
      return lines.join('\n') + `\n总计: ${commits.length} 条提交`;
    }
    if (format === 'csv') {
      const header = 'hash,date,author,email,message';
      const rows = commits.map(c => {
        const ds = fmtDate(c.date);
        return `"${c.hash}","${ds}","${c.author_name}","${c.author_email}","${c.message.replace(/"/g, '""').replace(/\r?\n/g, ' ')}"`;
      });
      return [header, ...rows].join('\n');
    }
    if (format === 'json') {
      return JSON.stringify(commits, null, 2);
    }
    if (format === 'md') {
      const lines = commits.map(c => {
        const ds = fmtDate(c.date);
        return `- \`${c.hash.slice(0,7)}\` | ${ds} | **${c.author_name}** | ${c.message}`;
      });
      return `# Git 提交记录\n\n总计: ${commits.length} 条\n\n${lines.join('\n')}`;
    }
  }

  const headerLines = [
    `Git 导出报告`,
    `项目路径: ${projectPath.value}`,
    `生成时间: ${now}`,
    ``,
    `选定的作者:`,
  ];
  if (selectedAuthors.value.length === 0) {
    headerLines.push(`- 全部作者 (共 ${authors.value.length} 人)`);
  } else {
    selectedAuthors.value.forEach(author => {
      headerLines.push(`- ${author.name} <${author.email}> (共${author.count}次提交)`);
    });
  }
  headerLines.push(`日期筛选条件:`);
  if (dateMode.value === 'single' && singleDate.value) {
    headerLines.push(`- 单个日期: ${singleDate.value}`);
  } else {
    dateRanges.value.filter(r => r.start && r.end).forEach(r => {
      headerLines.push(`- ${r.start} 至 ${r.end}`);
    });
  }

  if (format === 'txt') {
    const bodyLines = commits.map(c => {
      const ds = fmtDate(c.date);
      return `${c.hash} - ${ds} - ${c.author_name}(${c.author_email}) : ${c.message.replace(/\r?\n/g, ' ')}`;
    });
    return [...headerLines, '提交详情:', ...bodyLines, `总计: ${commits.length} 条提交`].join('\n');
  }

  if (format === 'csv') {
    const header = 'hash,date,author_name,author_email,message';
    const rows = commits.map(c => {
      const ds = fmtDate(c.date);
      return `"${c.hash}","${ds}","${c.author_name}","${c.author_email}","${c.message.replace(/"/g, '""').replace(/\r?\n/g, ' ')}"`;
    });
    return [header, ...rows].join('\n');
  }

  if (format === 'json') {
    const data = {
      project_path: projectPath.value,
      generated_at: now,
      selected_authors: selectedAuthors.value.length > 0
        ? selectedAuthors.value.map(a => ({ name: a.name, email: a.email, total_commits: a.count }))
        : authors.value.map(a => ({ name: a.name, email: a.email, total_commits: a.count })),
      date_filters: (() => {
        if (dateMode.value === 'single' && singleDate.value) return [singleDate.value];
        return dateRanges.value.filter(r => r.start && r.end).map(r => ({ start: r.start, end: r.end }));
      })(),
      commits: commits,
      total: commits.length,
    };
    return JSON.stringify(data, null, 2);
  }

  if (format === 'md') {
    const bodyLines = commits.map(c => {
      const ds = fmtDate(c.date);
      return `- \`${c.hash.slice(0,7)}\` | ${ds} | **${c.author_name}**<${c.author_email}> | ${c.message.replace(/\r?\n/g, ' ')}`;
    });
    return [...headerLines, '## 提交详情', ...bodyLines, `**总计: ${commits.length} 条提交**`].join('\n');
  }

  return '';
}

function getDirFromFilePath(filePath: string): string {
  const idx = Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\'));
  return idx >= 0 ? filePath.slice(0, idx) : filePath;
}

async function openDirectory(dirPath: string) {
  if (!dirPath) return;
  try {
    await invoke('open_directory', { dirPath });
  } catch (e) {
    console.error('打开目录失败:', e);
    alert(`打开目录失败: ${e}`);
  }
}

function openRecordDirectory(record: ExportRecord) {
  if (record.file_path) {
    openDirectory(getDirFromFilePath(record.file_path));
  }
}

async function saveHistoryRecord(filename: string, commitCount: number, filePath: string) {
  const record: ExportRecord = {
    timestamp: new Date().toLocaleString('zh-CN'),
    project_path: projectPath.value,
    filename,
    format: exportFormat.value,
    author_count: getEffectiveAuthorCount(),
    commit_count: commitCount,
    incremental: incrementalExport.value,
    file_path: filePath,
  };
  try {
    await invoke('save_export_record', { record });
    exportHistory.value.unshift(record);
  } catch (e) {
    console.error('保存历史记录失败:', e);
  }
}

const clearHistory = async () => {
  if (!confirm('确定清空导出历史？')) return;
  try {
    await invoke('clear_export_history');
    exportHistory.value = [];
  } catch (e) {
    console.error('清空历史失败:', e);
  }
};

// 预览：拉取数据但不导出
const previewData = async () => {
  const apiDateRanges: {start: string | null, end: string | null}[] = [];
  if (dateMode.value === 'single' && singleDate.value) {
    apiDateRanges.push({ start: singleDate.value, end: singleDate.value });
  } else if (dateMode.value === 'range') {
    dateRanges.value.filter(r => r.start && r.end).forEach(r => {
      apiDateRanges.push({ start: r.start, end: r.end });
    });
  }

  if (apiDateRanges.length === 0 && !incrementalExport.value) {
    alert('请选择有效的日期范围');
    return;
  }

  previewLoading.value = true;
  try {
    const authorEmails = getAuthorEmailsForQuery();
    let commits: GitCommit[];

    if (incrementalExport.value) {
      let lastExportTime: string | undefined;
      if (exportHistory.value.length > 0) {
        const last = exportHistory.value.find(r => r.project_path === projectPath.value);
        if (last) {
          const lastDate = new Date(last.timestamp.replace('年','/').replace('月','/').replace('日',' ').replace(':','/').replace(':','/'));
          lastExportTime = lastDate.toISOString();
        }
      }
      commits = await invoke('get_git_commits_for_incremental', {
        projectPath: projectPath.value,
        incrementalSince: lastExportTime,
        authorEmails,
      });
    } else {
      commits = await invoke('get_git_commits_in_date_range', {
        projectPath: projectPath.value,
        dateRanges: apiDateRanges,
        authorEmails,
      });
    }

    previewCommits.value = commits;
    resetExportFilename();
  } catch (error) {
    console.error('预览失败:', error);
    alert(`预览失败: ${error}`);
  } finally {
    previewLoading.value = false;
  }
};

const exportData = async () => {
  if (!previewCommits.value.length) {
    alert('请先点击"预览数据"加载提交记录');
    return;
  }

  const filename = sanitizeFilename(exportFilename.value);
  if (!filename) {
    alert('请输入有效的文件名称');
    return;
  }

  try {
    const content = buildReportContent(previewCommits.value, exportFormat.value, exportTemplate.value);

    let filePath: string | null;
    if (exportDir.value) {
      const fullPath = exportDir.value.endsWith('/') ? exportDir.value + filename : exportDir.value + '/' + filename;
      filePath = fullPath;
      await invoke('write_export_file', { path: fullPath, content });
    } else {
      const chosenPath = await save({
        filters: [{
          name: exportFormat.value.toUpperCase(),
          extensions: [exportFormat.value]
        }],
        defaultPath: filename
      });
      if (chosenPath) {
        await invoke('write_export_file', { path: chosenPath, content });
        filePath = chosenPath;
      }
    }

    if (filePath) {
      await saveHistoryRecord(filename, previewCommits.value.length, filePath);

      successFilePath.value = filePath;
      successDirPath.value = exportDir.value || getDirFromFilePath(filePath);
      showSuccessDialog.value = true;
    }
  } catch (error) {
    console.error('导出失败:', error);
    alert(`导出失败: ${error}`);
  }
};

const handleOpenDirectory = async () => {
  await openDirectory(successDirPath.value);
  showSuccessDialog.value = false;
};
</script>

<style scoped>
.git-analyzer {
  min-height: 100vh;
  min-width: 600px;
  max-width: 1000px;
  width: 100%;
  margin: 0 auto;
  overflow-x: hidden;
  background: linear-gradient(160deg, #f0f4f8 0%, #e8edf3 100%);
  color: #303133;
}

.page-header {
  padding: 24px 20px 8px;
  text-align: center;
}

.page-title {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: #1d2129;
}

.page-subtitle {
  margin: 8px 0 0;
  font-size: 13px;
  color: #909399;
}

.page-main {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 8px 20px 16px;
}

.page-footer {
  padding: 8px 20px 16px;
  text-align: center;
  font-size: 12px;
  color: #a8abb2;
}

/* 卡片 */
.section-card {
  border: 1px solid #ebeef5;
  border-radius: 10px;
  background: #fff;
}

.section-card :deep(.el-card__header) {
  padding: 12px 16px;
  border-bottom: 1px solid #f0f2f5;
  background: #fafbfc;
}

.section-card :deep(.el-card__body) {
  padding: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.section-header__title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  color: #303133;
}

.section-header__icon {
  font-size: 17px;
  color: #409eff;
}

.section-header__actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

/* 工具栏行 */
.toolbar-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.toolbar-row + .toolbar-row {
  margin-top: 12px;
}

.toolbar-row--compact {
  gap: 8px;
}

.toolbar-grow {
  flex: 1 1 180px;
  min-width: 0;
}

.toolbar-btn {
  flex-shrink: 0;
  height: 36px;
  padding: 0 14px;
}

.toolbar-btn--icon {
  width: 36px;
  padding: 0;
}

/* 表单 */
.uniform-form :deep(.el-form-item) {
  margin-bottom: 14px;
}

.uniform-form :deep(.el-form-item:last-child) {
  margin-bottom: 0;
}

.uniform-form :deep(.el-form-item__label) {
  padding-bottom: 6px;
  line-height: 1.4;
  font-size: 13px;
  color: #606266;
}

.form-hint {
  margin: 6px 0 0;
  font-size: 12px;
  line-height: 1.5;
  color: #909399;
}

.section-card :deep(.el-select.w-full) + .form-hint,
.section-card :deep(.el-select.w-full) + .scroll-panel {
  margin-top: 12px;
}

/* 导出设置网格 */
.export-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.export-col {
  min-width: 0;
}

/* 日期 */
.mode-switch {
  margin-bottom: 14px;
}

.date-range-block {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.range-separator {
  flex-shrink: 0;
  color: #909399;
  font-size: 13px;
}

/* 操作栏 */
.action-card :deep(.el-card__body) {
  padding: 14px 16px;
}

.action-bar {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.action-btn {
  flex: 1;
  max-width: 240px;
  height: 40px;
  font-size: 14px;
}

/* 滚动面板 */
.scroll-panel {
  border: 1px solid #ebeef5;
  border-radius: 8px;
  background: #fafbfc;
  overflow-y: auto;
}

.scroll-panel__label {
  margin-bottom: 8px;
  font-size: 12px;
  color: #909399;
}

.author-tags-scroll {
  max-height: 160px;
  margin-top: 12px;
  padding: 10px 12px;
}

.preview-panel {
  max-height: 320px;
  padding: 12px 14px;
}

.preview-text {
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  color: #606266;
  white-space: pre-wrap;
  word-break: break-word;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag-list__item {
  max-width: 100%;
}

.tag-list__item :deep(.el-tag__content) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 260px;
}

/* 历史记录 */
.history-timeline {
  padding-left: 4px;
}

.history-item {
  padding: 10px 12px;
  border: 1px solid #ebeef5;
  border-radius: 8px;
  background: #fafbfc;
}

.history-item__head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.history-item__name {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
  word-break: break-all;
}

.history-item__path {
  margin: 6px 0 0;
}

.history-item__link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #409eff;
  cursor: pointer;
}

.history-item__link:hover {
  text-decoration: underline;
}

/* 弹窗 */
.dialog-body {
  font-size: 14px;
  line-height: 1.6;
  color: #606266;
}

.dialog-body p {
  margin: 0;
  word-break: break-all;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

/* Element Plus 统一控件高度 */
.git-analyzer :deep(.el-input__wrapper),
.git-analyzer :deep(.el-select__wrapper) {
  min-height: 36px;
}

.git-analyzer :deep(.el-date-editor.el-input) {
  width: 100%;
}

.git-analyzer :deep(.el-select) {
  width: 100%;
}

.git-analyzer :deep(.el-radio-button__inner) {
  height: 36px;
  line-height: 34px;
  padding: 0 16px;
}

/* 作者下拉选项 */
.author-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-width: 0;
  width: 100%;
}

.author-option-main {
  display: flex;
  align-items: center;
  min-width: 0;
  flex: 1;
}

.author-option-check {
  flex-shrink: 0;
  margin-right: 8px;
}

.author-option-text {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.author-option-tag {
  flex-shrink: 0;
}

/* 滚动条 */
.scroll-panel::-webkit-scrollbar,
.author-tags-scroll::-webkit-scrollbar {
  width: 6px;
}

.scroll-panel::-webkit-scrollbar-thumb,
.author-tags-scroll::-webkit-scrollbar-thumb {
  background: #c0c4cc;
  border-radius: 3px;
}

.scroll-panel::-webkit-scrollbar-thumb:hover,
.author-tags-scroll::-webkit-scrollbar-thumb:hover {
  background: #909399;
}

@media (max-width: 720px) {
  .export-grid {
    grid-template-columns: 1fr;
  }

  .action-bar {
    flex-direction: column;
  }

  .action-btn {
    max-width: none;
    width: 100%;
  }
}
</style>

<style>
.app-select-dropdown.el-popper {
  box-sizing: border-box;
}
.app-select-dropdown .el-select-dropdown__wrap {
  max-width: 100%;
}
.app-select-dropdown .el-select-dropdown__item {
  overflow: hidden;
}
.app-select-dropdown .el-select-dropdown__item > span {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.project-path-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  min-width: 0;
}
.project-path-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.project-path-delete {
  flex-shrink: 0;
  color: #909399;
  font-size: 14px;
  padding: 2px;
  border-radius: 4px;
}
.project-path-delete:hover {
  color: #f56c6c;
  background: #fef0f0;
}
.project-path-footer {
  padding: 4px 8px;
  border-top: 1px solid #ebeef5;
  text-align: center;
}
.project-path-select .el-select-dropdown__item {
  padding-right: 8px;
}
</style>
