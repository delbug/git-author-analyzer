<template>
  <div class="git-analyzer p-6 max-w-4xl mx-auto">
    <h1 class="text-2xl font-bold text-center mb-8 text-gray-800">
      <el-icon class="mr-2"><Files /></el-icon>
      Git 导出器
    </h1>

    <!-- 配置导入 -->
    <el-card class="mb-6" shadow="hover">
      <template #header>
        <div class="flex items-center">
          <el-icon><Collection /></el-icon>
          <span class="ml-2 font-medium">上次配置</span>
        </div>
      </template>
      <div class="flex gap-3 items-center">
        <el-select
          v-model="selectedConfigIndex"
          placeholder="选择上次配置"
          class="flex-1"
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
        <el-button size="small" type="info" @click="saveCurrentAsConfig">
          保存当前为配置
        </el-button>
      </div>
    </el-card>

    <!-- 项目路径 -->
    <el-card class="mb-6" shadow="hover">
      <template #header>
        <div class="flex items-center">
          <el-icon><FolderOpened /></el-icon>
          <span class="ml-2 font-medium">项目路径</span>
        </div>
      </template>
      <div class="flex gap-3">
        <el-input
          v-model="projectPath"
          placeholder="输入或选择本地 Git 项目目录"
          clearable
          class="flex-1"
        >
          <template #prefix>
            <el-icon><Monitor /></el-icon>
          </template>
        </el-input>
        <el-button type="primary" @click="browsePath" :icon="Search">
          选择目录
        </el-button>
        <el-button type="success" @click="loadAuthors" :loading="loading" :disabled="!projectPath" :icon="Refresh">
          {{ loading ? '加载中...' : '加载作者' }}
        </el-button>
      </div>
    </el-card>

    <!-- 作者选择 -->
    <el-card v-if="authors.length > 0" class="mb-6" shadow="hover">
      <template #header>
        <div class="flex items-center justify-between">
          <div class="flex items-center">
            <el-icon><User /></el-icon>
            <span class="ml-2 font-medium">选择作者</span>
            <el-tag v-if="selectedAuthors.length" type="primary" size="small" class="ml-2">
              已选 {{ selectedAuthors.length }}/{{ authors.length }}
            </el-tag>
          </div>
          <div>
            <el-button size="small" @click="selectAllAuthors">全选</el-button>
            <el-button size="small" @click="selectNoneAuthors">清空</el-button>
          </div>
        </div>
      </template>

      <!-- 搜索 + 多选下拉（不收起） -->
      <el-select
        v-model="selectedAuthorUids"
        multiple
        filterable
        reserve-keyword
        filter-method="filterAuthors"
        remote
        :remote-method="filterAuthors"
        placeholder="输入作者名称搜索，点击添加到已选"
        class="w-full mb-3"
        clearable
        @change="handleAuthorChange"
      >
        <el-option
          v-for="author in filteredAuthors"
          :key="author._uid"
          :label="`${author.name}(${author.email})`"
          :value="author._uid"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center">
              <span class="mr-2">{{ isSelectedByUid(author._uid) ? '✅' : '⬜' }}</span>
              <span>{{ author.name}}({{ author.email }})</span>
            </div>
            <el-tag size="small">{{ author.count }} commits</el-tag>
          </div>
        </el-option>
      </el-select>

      <!-- 已选作者标签（可滚动） -->
      <div v-if="selectedAuthors.length" class="max-h-40 overflow-y-auto">
        <el-tag
          v-for="a in selectedAuthors"
          :key="a._uid"
          closable
          type="primary"
          size="small"
          class="mr-2 mb-1"
          @close="removeAuthor(a)"
        >
          {{ a.name}}({{ a.email }})
        </el-tag>
      </div>
    </el-card>

    <!-- 日期范围 -->
    <el-card class="mb-6" shadow="hover">
      <template #header>
        <div class="flex items-center">
          <el-icon><Calendar /></el-icon>
          <span class="ml-2 font-medium">日期范围</span>
        </div>
      </template>
      <el-radio-group v-model="dateMode" class="mb-4">
        <el-radio-button value="single">单个日期</el-radio-button>
        <el-radio-button value="range">时间段</el-radio-button>
      </el-radio-group>

      <div v-if="dateMode === 'single'" class="flex gap-3 items-center">
        <el-date-picker
          v-model="singleDate"
          type="date"
          placeholder="选择日期"
          format="YYYY-MM-DD"
          value-format="YYYY-MM-DD"
          class="flex-1"
          clearable
        />
      </div>

      <div v-else>
        <div v-for="(range, index) in dateRanges" :key="index" class="flex items-center gap-2 mb-2">
          <el-date-picker
            v-model="range.start"
            type="date"
            placeholder="开始日期"
            format="YYYY-MM-DD"
            value-format="YYYY-MM-DD"
            class="flex-1"
            clearable
          />
          <span class="text-gray-400">至</span>
          <el-date-picker
            v-model="range.end"
            type="date"
            placeholder="结束日期"
            format="YYYY-MM-DD"
            value-format="YYYY-MM-DD"
            class="flex-1"
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
              <el-button type="danger" :icon="Delete" circle size="small" />
            </template>
          </el-popconfirm>
        </div>
        <el-button type="primary" plain :icon="Plus" @click="addRange">
          添加时间段
        </el-button>
      </div>
    </el-card>

    <!-- 导出设置 -->
    <el-card class="mb-6" shadow="hover">
      <template #header>
        <div class="flex items-center">
          <el-icon><Setting /></el-icon>
          <span class="ml-2 font-medium">导出设置</span>
        </div>
      </template>
      <el-row :gutter="20">
        <el-col :span="12">
          <el-form label-position="top" size="default">
            <el-form-item label="导出格式">
              <el-select v-model="exportFormat" class="w-full">
                <el-option label="TXT 文本" value="txt" />
                <el-option label="CSV 表格" value="csv" />
                <el-option label="JSON 数据" value="json" />
                <el-option label="Markdown" value="md" />
              </el-select>
            </el-form-item>
            <el-form-item label="导出模板">
              <el-select v-model="exportTemplate" class="w-full">
                <el-option label="完整版（含作者信息）" value="full" />
                <el-option label="精简版（仅提交记录）" value="compact" />
              </el-select>
            </el-form-item>
            <el-form-item label="增量导出">
              <el-switch v-model="incrementalExport" />
            </el-form-item>
          </el-form>
        </el-col>
        <el-col :span="12">
          <el-form label-position="top" size="default">
            <el-form-item label="导出目录">
              <div class="flex gap-2">
                <el-input
                  v-model="exportDir"
                  placeholder="选择导出目录"
                  readonly
                  clearable
                  class="flex-1"
                >
                  <template #prefix>
                    <el-icon><Folder /></el-icon>
                  </template>
                </el-input>
                <el-button @click="browseExportDir" :icon="Search">选择</el-button>
              </div>
            </el-form-item>
          </el-form>
        </el-col>
      </el-row>
    </el-card>

    <!-- 操作按钮 -->
    <div class="flex justify-center gap-4 mb-8">
      <el-button
        type="warning"
        size="large"
        :disabled="selectedAuthors.length === 0 || !hasValidDateFilter()"
        :loading="previewLoading"
        :icon="Document"
        @click="previewData"
      >
        {{ previewLoading ? '加载中...' : '预览数据' }}
      </el-button>
      <el-button
        type="primary"
        size="large"
        :disabled="selectedAuthors.length === 0 || !hasValidDateFilter() || !previewCommits.length"
        :icon="Download"
        @click="exportData"
      >
        导出为 {{ formatLabel }}
      </el-button>
    </div>

    <!-- 预览 -->
    <el-card v-if="previewCommits.length > 0" class="mb-6" shadow="hover">
      <template #header>
        <div class="flex items-center justify-between">
          <div class="flex items-center">
            <el-icon><Document /></el-icon>
            <span class="ml-2 font-medium">提交预览</span>
            <el-tag class="ml-2" type="success">{{ previewCommits.length }} 条</el-tag>
          </div>
          <el-button size="small" @click="previewCommits = []">收起</el-button>
        </div>
      </template>
      <div class="max-h-80 overflow-y-auto text-sm bg-gray-50 rounded p-3">
        <pre class="whitespace-pre-wrap text-gray-700">{{ previewText }}</pre>
      </div>
    </el-card>

    <!-- 导出历史 -->
    <el-card v-if="exportHistory.length > 0" class="mb-6" shadow="hover">
      <template #header>
        <div class="flex items-center justify-between">
          <div class="flex items-center">
            <el-icon><Clock /></el-icon>
            <span class="ml-2 font-medium">导出历史</span>
          </div>
          <el-button size="small" type="danger" plain @click="clearHistory">
            清空
          </el-button>
        </div>
      </template>
      <el-timeline>
        <el-timeline-item
          v-for="(record, idx) in exportHistory"
          :key="idx"
          :timestamp="record.timestamp"
          placement="top"
          type="primary"
        >
          <el-card shadow="never">
            <div class="flex items-center gap-2 text-sm">
              <el-tag size="small" :type="formatTagType(record.format)">
                {{ record.format.toUpperCase() }}
              </el-tag>
              <span class="font-medium">{{ record.filename }}</span>
              <el-tag v-if="record.incremental" size="small" type="warning">增量</el-tag>
            </div>
            <div class="text-xs text-gray-400 mt-1">
              {{ record.author_count }} 作者 · {{ record.commit_count }} 提交 · {{ record.project_path }}
            </div>
          </el-card>
        </el-timeline-item>
      </el-timeline>
    </el-card>

    <!-- 导出成功弹窗 -->
    <el-dialog
      v-model="showSuccessDialog"
      title="导出成功"
      width="400px"
      :icon="CircleCheckFilled"
    >
      <div class="text-sm text-gray-600 mb-4">
        <p class="mb-1"><strong>文件：</strong>{{ successFilePath }}</p>
      </div>
      <template #footer>
        <el-button type="primary" @click="handleOpenDirectory">
          <el-icon class="mr-1"><Folder /></el-icon>
          打开导出目录
        </el-button>
        <el-button @click="showSuccessDialog = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
  <div class="text-center text-xs text-gray-400 pb-2">
    v{{ appVersion }}
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import {
  Files, FolderOpened, Monitor, Search, Refresh,
  User, Calendar, Setting, Document, Download,
  Delete, Plus, Clock, Folder,
  CircleCheckFilled, Collection,
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
}

const projectPath = ref('');
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

// 保存当前为配置
function saveCurrentAsConfig() {
  const config: SavedConfig = {
    label: `${projectPath.value.split('/').filter(Boolean).pop() || '项目'} - ${new Date().toLocaleDateString('zh-CN')}`,
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
  saveConfigsList();
  alert('配置已保存！下次打开时可从下拉列表选择');
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
  selectedAuthors.value.forEach(author => {
    headerLines.push(`- ${author.name} <${author.email}> (共${author.count}次提交)`);
  });
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
      selected_authors: selectedAuthors.value.map(a => ({ name: a.name, email: a.email, total_commits: a.count })),
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

async function saveHistoryRecord(filename: string, commitCount: number) {
  const record: ExportRecord = {
    timestamp: new Date().toLocaleString('zh-CN'),
    project_path: projectPath.value,
    filename,
    format: exportFormat.value,
    author_count: selectedAuthors.value.length,
    commit_count: commitCount,
    incremental: incrementalExport.value,
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
  if (selectedAuthors.value.length === 0) {
    alert('请至少选择一个作者');
    return;
  }

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
    const authorEmails = selectedAuthors.value.map(a => a.email);
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
  } catch (error) {
    console.error('预览失败:', error);
    alert(`预览失败: ${error}`);
  } finally {
    previewLoading.value = false;
  }
};

const exportData = async () => {
  if (selectedAuthors.value.length === 0) {
    alert('请至少选择一个作者');
    return;
  }

  if (!previewCommits.value.length) {
    alert('请先点击"预览数据"加载提交记录');
    return;
  }

  try {
    const baseName = projectPath.value.split('/').filter(Boolean).pop() || 'repo';
    const filename = getFilename(baseName);
    const content = buildReportContent(previewCommits.value, exportFormat.value, exportTemplate.value);

    let filePath: string | null;
    if (exportDir.value) {
      // 用户指定了导出目录，通过 Rust 后端写入
      const fullPath = exportDir.value.endsWith('/') ? exportDir.value + filename : exportDir.value + '/' + filename;
      filePath = fullPath;
      await invoke('write_export_file', { path: fullPath, content });
    } else {
      // 未指定目录，弹出保存对话框
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
      await saveHistoryRecord(filename, previewCommits.value.length);

      successFilePath.value = filePath;
      successDirPath.value = exportDir.value || (typeof filePath === 'string' ? filePath.substring(0, filePath.lastIndexOf('/')) : '');
      showSuccessDialog.value = true;
    }
  } catch (error) {
    console.error('导出失败:', error);
    alert(`导出失败: ${error}`);
  }
};

const handleOpenDirectory = async () => {
  if (successDirPath.value) {
    try {
      await invoke('open_directory', { dirPath: successDirPath.value });
    } catch (e) {
      console.error('打开目录失败:', e);
      alert(`打开目录失败: ${e}`);
    }
  }
  showSuccessDialog.value = false;
};
</script>

<style scoped>
.git-analyzer {
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e9f2 100%);
}
.el-card {
  border-radius: 12px;
}
.el-card :deep(.el-card__header) {
  padding: 12px 16px;
  background: #fafafa;
}
</style>
