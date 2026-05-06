<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { DocumentCopy, Delete, Setting, Plus, Close, Top, Link, Aim } from "@element-plus/icons-vue";

const isAlwaysOnTop = ref(false);
const isTauri = typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);

// ──────────── 吸附功能 ────────────
const snapEnabled = ref(false);
const snapPosition = ref<'Left' | 'Right' | 'Top' | 'Bottom'>('Right');
const snapGap = ref(0);
const snapTargetFound = ref(false);
const snapTargetTitle = ref('');
const showSnapPopover = ref(false);
const SNAP_TARGET_KEYWORD = '闲鱼管家';
let snapStatusTimer: ReturnType<typeof setInterval> | null = null;

async function toggleSnap(enable: boolean) {
  if (!isTauri) {
    ElMessage.warning('吸附功能仅支持桌面端');
    return;
  }
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    if (enable) {
      await invoke('start_snap', {
        config: {
          position: snapPosition.value,
          target_keyword: SNAP_TARGET_KEYWORD,
          gap: snapGap.value,
        },
      });
      snapEnabled.value = true;
      startSnapStatusPolling();
      ElMessage.success('已开启吸附');
    } else {
      await invoke('stop_snap');
      snapEnabled.value = false;
      snapTargetFound.value = false;
      snapTargetTitle.value = '';
      stopSnapStatusPolling();
      ElMessage.success('已关闭吸附');
    }
  } catch (e: any) {
    ElMessage.error('吸附操作失败: ' + (e?.message || e));
  }
}

async function updateSnapStatus() {
  if (!isTauri || !snapEnabled.value) return;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const status = await invoke<{
      enabled: boolean;
      target_found: boolean;
      target_title: string | null;
      position: string;
    }>('get_snap_status');
    snapEnabled.value = status.enabled;
    snapTargetFound.value = status.target_found;
    snapTargetTitle.value = status.target_title || '';
  } catch {
    // 静默
  }
}

function startSnapStatusPolling() {
  stopSnapStatusPolling();
  snapStatusTimer = setInterval(updateSnapStatus, 2000);
}

function stopSnapStatusPolling() {
  if (snapStatusTimer) {
    clearInterval(snapStatusTimer);
    snapStatusTimer = null;
  }
}

async function handleSnapPositionChange() {
  if (snapEnabled.value) {
    await toggleSnap(true);
  }
}

async function handleSnapGapChange() {
  if (snapEnabled.value) {
    await toggleSnap(true);
  }
}

async function toggleAlwaysOnTop() {
  isAlwaysOnTop.value = !isAlwaysOnTop.value;
  if (isTauri) {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const win = getCurrentWindow();
      console.log('setAlwaysOnTop =>', isAlwaysOnTop.value, 'window label:', win.label);
      await win.setAlwaysOnTop(isAlwaysOnTop.value);
      console.log('setAlwaysOnTop success');
    } catch (e) {
      console.error('setAlwaysOnTop failed:', e);
      ElMessage.error('置顶失败: ' + (e instanceof Error ? e.message : String(e)));
      isAlwaysOnTop.value = !isAlwaysOnTop.value;
      return;
    }
  }
  ElMessage.success(isAlwaysOnTop.value ? '已置顶' : '取消置顶');
}
import {
  type RoundMode,
  type Brand,
  type HistoryRecord,
  calculate,
  matchTierRate,
  generateQuote,
  getHistory,
  saveHistory,
  clearHistory,
  getSettings,
  saveSettings,
  getActiveBrand,
  createBrand,
} from "./utils/calculator";

const APP_VERSION = "1.1.1";

const originalAmount = ref<number | undefined>(undefined);
const receivedRate = ref<number | undefined>(undefined);
const expenseRate = ref<number | undefined>(undefined);
const roundMode = ref<RoundMode>("none");
const feeRate = ref<number>(1.6);

const historyList = ref<HistoryRecord[]>([]);
const originalAmountRef = ref<InstanceType<typeof import('element-plus')['ElInputNumber']> | null>(null);
const settings = ref(getSettings());
const activeBrandId = ref(settings.value.activeBrandId);

const currentBrand = computed(() => getActiveBrand(settings.value));

function applyBrand(brand: Brand) {
  receivedRate.value = brand.receivedRate || undefined;
  expenseRate.value = brand.expenseRate || undefined;
  feeRate.value = brand.feeRate;
  roundMode.value = brand.roundMode;
}

function focusOriginalAmount() {
  nextTick(() => {
    const el = originalAmountRef.value?.$el?.querySelector('input');
    if (el) {
      el.focus();
      el.select();
    }
  });
}

async function checkForUpdates() {
  if (!isTauri) return;
  try {
    const { check } = await import('@tauri-apps/plugin-updater');
    const update = await check();
    if (update) {
      const confirmed = await ElMessageBox.confirm(
        `发现新版本 v${update.version}，是否立即更新？`,
        '版本更新',
        { confirmButtonText: '立即更新', cancelButtonText: '稍后' }
      ).catch(() => false);
      if (confirmed) {
        await update.downloadAndInstall();
        const { relaunch } = await import('@tauri-apps/plugin-process');
        await relaunch();
      }
    }
  } catch {
    // 静默失败
  }
}

onMounted(() => {
  settings.value = getSettings();
  activeBrandId.value = settings.value.activeBrandId;
  const brand = getActiveBrand(settings.value);
  applyBrand(brand);
  historyList.value = getHistory();
  focusOriginalAmount();
  setTimeout(checkForUpdates, 3000);
  // 恢复吸附状态
  updateSnapStatus().then(() => {
    if (snapEnabled.value) startSnapStatusPolling();
  });
});

function switchBrand(brandId: string) {
  activeBrandId.value = brandId;
  settings.value.activeBrandId = brandId;
  saveSettings(settings.value);
  const brand = getActiveBrand(settings.value);
  applyBrand(brand);
  originalAmount.value = undefined;
  focusOriginalAmount();
}

function handleOriginalAmountEnter() {
  if (result.value && quoteText.value) {
    handleCopyQuote();
  }
}

function handleGlobalKeydown(e: KeyboardEvent) {
  const isCtrlOrCmd = e.ctrlKey || e.metaKey;

  if (e.key === 'Escape') {
    if (showSettings.value) {
      showSettings.value = false;
      return;
    }
    originalAmount.value = undefined;
    focusOriginalAmount();
    return;
  }

  if (isCtrlOrCmd && e.key === 'n') {
    e.preventDefault();
    originalAmount.value = undefined;
    focusOriginalAmount();
    return;
  }

  if (isCtrlOrCmd && e.key === 'c') {
    const selection = window.getSelection()?.toString();
    if (!selection && result.value && quoteText.value) {
      e.preventDefault();
      handleCopyQuote();
    }
  }
}

const activeBrandName = computed(() => {
  const brand = settings.value.brands.find(b => b.id === activeBrandId.value);
  return brand?.name || '默认品牌';
});

const brandConfigured = computed(() => {
  return !!receivedRate.value && receivedRate.value > 0;
});

const showSettings = ref(false);
const editingBrandIndex = ref(0);

const editingBrand = computed({
  get: () => settings.value.brands[editingBrandIndex.value] || settings.value.brands[0],
  set: (val: Brand) => {
    settings.value.brands[editingBrandIndex.value] = val;
  },
});

function openSettings() {
  settings.value = getSettings();
  const idx = settings.value.brands.findIndex(b => b.id === activeBrandId.value);
  editingBrandIndex.value = idx >= 0 ? idx : 0;
  showSettings.value = true;
}

function switchEditBrand(index: number) {
  editingBrandIndex.value = index;
}

async function addBrand() {
  try {
    const { value: name } = await ElMessageBox.prompt("请输入品牌名称", "新增品牌", {
      confirmButtonText: "确定",
      cancelButtonText: "取消",
      inputPattern: /.+/,
      inputErrorMessage: "品牌名称不能为空",
    });
    const brand = createBrand(name);
    settings.value.brands.push(brand);
    editingBrandIndex.value = settings.value.brands.length - 1;
  } catch {
    // cancelled
  }
}

function removeBrand(index: number) {
  if (settings.value.brands.length <= 1) {
    ElMessage.warning("至少保留一个品牌");
    return;
  }
  const removed = settings.value.brands.splice(index, 1)[0];
  if (editingBrandIndex.value >= settings.value.brands.length) {
    editingBrandIndex.value = settings.value.brands.length - 1;
  }
  if (settings.value.activeBrandId === removed.id) {
    settings.value.activeBrandId = settings.value.brands[0].id;
  }
}

function addTier(brandIndex: number) {
  const brand = settings.value.brands[brandIndex];
  if (!brand.tiers) brand.tiers = [];
  const lastMax = brand.tiers.length > 0 ? brand.tiers[brand.tiers.length - 1].maxAmount : 0;
  brand.tiers.push({
    minAmount: lastMax > 0 ? lastMax + 1 : 1,
    maxAmount: 0,
    receivedRate: brand.receivedRate || 65,
  });
}

function removeTier(brandIndex: number, tierIndex: number) {
  const brand = settings.value.brands[brandIndex];
  if (brand.tiers) brand.tiers.splice(tierIndex, 1);
}

function handleSettingsSave() {
  saveSettings(settings.value);
  activeBrandId.value = settings.value.activeBrandId;
  const brand = getActiveBrand(settings.value);
  applyBrand(brand);
  showSettings.value = false;
  ElMessage.success("设置已保存");
}

const effectiveReceivedRate = computed(() => {
  const brand = currentBrand.value;
  if (brand.useTieredRate && brand.tiers && brand.tiers.length > 0 && originalAmount.value) {
    const tierRate = matchTierRate(originalAmount.value, brand.tiers);
    if (tierRate !== null) return tierRate;
  }
  return receivedRate.value || 0;
});

const matchedTierLabel = computed(() => {
  const brand = currentBrand.value;
  if (!brand.useTieredRate || !brand.tiers || !originalAmount.value) return '';
  const tier = brand.tiers.find(t => {
    const inMin = originalAmount.value! >= t.minAmount;
    const inMax = t.maxAmount === 0 || originalAmount.value! <= t.maxAmount;
    return inMin && inMax;
  });
  if (!tier) return '未匹配区间';
  const max = tier.maxAmount === 0 ? '∞' : tier.maxAmount;
  return `${tier.minAmount}-${max}元 → ${tier.receivedRate}%`;
});

const result = computed(() => {
  if (!originalAmount.value || !effectiveReceivedRate.value) return null;
  return calculate({
    originalAmount: originalAmount.value,
    receivedRate: effectiveReceivedRate.value,
    roundMode: roundMode.value,
    feeRate: feeRate.value / 100,
    expenseRate: expenseRate.value || 0,
  });
});

const profitColor = computed(() => {
  if (!result.value) return "#333";
  return result.value.netProfit >= 0 ? "#67c23a" : "#f56c6c";
});

const quoteText = computed(() => {
  if (!result.value || !originalAmount.value) return "";
  return generateQuote(originalAmount.value, result.value.receivedAmount);
});

function playSuccessSound() {
  try {
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.frequency.setValueAtTime(880, ctx.currentTime);
    osc.frequency.setValueAtTime(1100, ctx.currentTime + 0.08);
    gain.gain.setValueAtTime(0.15, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.2);
    osc.start(ctx.currentTime);
    osc.stop(ctx.currentTime + 0.2);
  } catch {
    // 静默失败
  }
}

async function handleCopyQuote() {
  if (!quoteText.value) return;
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(quoteText.value);
    } else {
      const textarea = document.createElement("textarea");
      textarea.value = quoteText.value;
      textarea.style.cssText = "position:fixed;left:-9999px;top:-9999px;opacity:0";
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
    }
    playSuccessSound();
    ElMessage.success("已复制报价");
  } catch {
    ElMessage.error("复制失败，请手动复制");
  }
}

let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;

watch(result, (val) => {
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
  if (!val || !originalAmount.value || !receivedRate.value) return;
  autoSaveTimer = setTimeout(() => {
    historyList.value = saveHistory({
      brandName: activeBrandName.value,
      originalAmount: originalAmount.value!,
      receivedRate: receivedRate.value!,
      expenseRate: expenseRate.value || 0,
      roundMode: roundMode.value,
      feeRate: feeRate.value,
      receivedAmount: val.receivedAmount,
      expenseAmount: val.expenseAmount,
      serviceFee: val.serviceFee,
      netProfit: val.netProfit,
    });
  }, 1000);
});

onMounted(() => {
  document.addEventListener('keydown', handleGlobalKeydown);
});

onUnmounted(() => {
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
  document.removeEventListener('keydown', handleGlobalKeydown);
  stopSnapStatusPolling();
});

const historyExpanded = ref(false);

const displayHistory = computed(() => {
  if (historyExpanded.value) return historyList.value.slice(0, 100);
  return historyList.value.slice(0, 3);
});

function toggleHistory() {
  historyExpanded.value = !historyExpanded.value;
}

function handleClear() {
  historyList.value = clearHistory();
  historyExpanded.value = false;
  ElMessage.success("已清空");
}
</script>

<template>
  <div class="app-container">
    <!-- 标题栏 -->
    <div class="header-row">
      <span class="header-title">餐饮计算器</span>
      <div class="header-actions">
        <el-popover
          v-model:visible="showSnapPopover"
          placement="bottom-end"
          :width="280"
          trigger="click"
        >
          <template #reference>
            <el-button
              :type="snapEnabled ? 'primary' : 'default'"
              :icon="Link"
              circle
              size="small"
              :title="snapEnabled ? '吸附中 - 点击设置' : '窗口吸附'"
            />
          </template>
          <div class="snap-panel">
            <div class="snap-panel-title">吸附设置</div>
            <div class="snap-panel-row">
              <span class="snap-label">目标应用</span>
              <span class="snap-value">{{ SNAP_TARGET_KEYWORD }}</span>
            </div>
            <div class="snap-panel-row">
              <span class="snap-label">连接状态</span>
              <span v-if="snapEnabled && snapTargetFound" class="snap-status snap-status-ok">● 已连接</span>
              <span v-else-if="snapEnabled" class="snap-status snap-status-searching">◌ 搜索中...</span>
              <span v-else class="snap-status snap-status-off">○ 未启用</span>
            </div>
            <div v-if="snapTargetTitle" class="snap-panel-row">
              <span class="snap-label">窗口标题</span>
              <span class="snap-value snap-title-ellipsis">{{ snapTargetTitle }}</span>
            </div>
            <div class="snap-panel-row">
              <span class="snap-label">吸附位置</span>
              <el-radio-group v-model="snapPosition" size="small" @change="handleSnapPositionChange">
                <el-radio-button value="Left">左</el-radio-button>
                <el-radio-button value="Right">右</el-radio-button>
                <el-radio-button value="Top">上</el-radio-button>
                <el-radio-button value="Bottom">下</el-radio-button>
              </el-radio-group>
            </div>
            <div class="snap-panel-row">
              <span class="snap-label">间距</span>
              <el-input-number
                v-model="snapGap"
                :min="0"
                :max="50"
                :step="1"
                size="small"
                :controls="true"
                style="width: 120px"
                @change="handleSnapGapChange"
              />
              <span class="snap-unit">px</span>
            </div>
            <div class="snap-panel-actions">
              <el-button
                v-if="!snapEnabled"
                type="primary"
                size="small"
                @click="toggleSnap(true)"
                style="width: 100%"
              >开启吸附</el-button>
              <el-button
                v-else
                type="danger"
                size="small"
                @click="toggleSnap(false)"
                style="width: 100%"
              >关闭吸附</el-button>
            </div>
          </div>
        </el-popover>
        <el-button
          :type="isAlwaysOnTop ? 'warning' : 'default'"
          :icon="Top"
          circle
          size="small"
          @click="toggleAlwaysOnTop"
          :title="isAlwaysOnTop ? '取消置顶' : '窗口置顶'"
        />
        <el-button :icon="Setting" circle size="small" @click="openSettings" />
      </div>
    </div>

    <!-- 品牌切换 -->
    <div class="brand-tabs">
      <div
        v-for="brand in settings.brands"
        :key="brand.id"
        class="brand-tab"
        :class="{ active: brand.id === activeBrandId }"
        @click="switchBrand(brand.id)"
      >
        {{ brand.name }}
      </div>
    </div>

    <!-- 输入区 -->
    <div class="input-section">
      <template v-if="brandConfigured">
        <div class="brand-info-bar">
          <span class="brand-info-item">收款 <b>{{ effectiveReceivedRate }}%</b></span>
          <span v-if="expenseRate" class="brand-info-item">支出 <b>{{ expenseRate }}%</b></span>
          <span class="brand-info-item">费率 <b>{{ feeRate }}%</b></span>
          <span class="brand-info-item">{{ roundMode === 'none' ? '不抹零' : roundMode === 'fen' ? '去分' : '去毛' }}</span>
          <span v-if="matchedTierLabel" class="brand-info-item tier-matched"><el-icon style="vertical-align: -2px"><Aim /></el-icon> {{ matchedTierLabel }}</span>
        </div>
        <div class="input-row">
          <label>原价金额</label>
          <el-input-number
            ref="originalAmountRef"
            v-model="originalAmount"
            :min="0"
            :precision="2"
            :controls="false"
            placeholder="输入原价"
            size="large"
            class="input-number"
            @keydown.enter="handleOriginalAmountEnter"
          />
          <span class="unit">元</span>
        </div>
      </template>
      <div v-else class="no-config-tip">
        <p>当前品牌「{{ activeBrandName }}」尚未配置收款比例</p>
        <el-button type="primary" size="small" @click="openSettings">去设置</el-button>
      </div>
    </div>

    <!-- 结果区 -->
    <div v-if="result" class="result-section">
      <div class="result-item">
        <span class="result-label">收款金额</span>
        <span class="result-value">¥ {{ result.receivedAmount.toFixed(2) }}</span>
      </div>
      <div class="result-item">
        <span class="result-label">手续费</span>
        <span class="result-value">¥ {{ result.serviceFee.toFixed(2) }}</span>
      </div>
      <div class="result-item">
        <span class="result-label">支出金额</span>
        <span class="result-value">¥ {{ result.expenseAmount.toFixed(2) }}</span>
      </div>
      <div class="result-item result-profit">
        <span class="result-label">净 利 润</span>
        <span class="result-value profit-value" :style="{ color: profitColor }">
          {{ result.netProfit >= 0 ? '' : '-' }}¥ {{ Math.abs(result.netProfit).toFixed(2) }}
        </span>
      </div>

      <div class="quote-section">
        <div class="quote-text">{{ quoteText }}</div>
        <el-button type="primary" :icon="DocumentCopy" @click="handleCopyQuote">复制报价</el-button>
      </div>
    </div>

    <div v-else-if="brandConfigured" class="result-section result-empty">
      <span>请输入原价金额</span>
    </div>

    <!-- 历史记录 -->
    <div v-if="historyList.length" class="history-section">
      <div class="history-header" @click="toggleHistory">
        <span class="history-title">
          历史记录（{{ historyList.length }} 条）
          <span class="history-toggle">{{ historyExpanded ? '收起' : '展开' }}</span>
        </span>
        <el-button size="small" text type="danger" :icon="Delete" @click.stop="handleClear">清空</el-button>
      </div>
      <div class="history-list">
        <div v-for="item in displayHistory" :key="item.id" class="history-item">
          <span class="history-main">
            <span v-if="item.brandName" class="history-brand">{{ item.brandName }}</span>
            原价{{ item.originalAmount }} → 收{{ item.receivedAmount }} →
            <span :style="{ color: item.netProfit >= 0 ? '#67c23a' : '#f56c6c' }">
              净利{{ item.netProfit }}
            </span>
          </span>
          <span class="history-time">{{ item.time }}</span>
        </div>
      </div>
      <div v-if="!historyExpanded && historyList.length > 3" class="history-more" @click="toggleHistory">
        查看全部 {{ historyList.length }} 条记录
      </div>
    </div>

    <!-- 底部版权 -->
    <div class="footer">
      <span>v{{ APP_VERSION }}</span>
      <span>山东发傲网络科技有限公司赞助开发</span>
    </div>

    <!-- 设置弹窗 -->
    <el-dialog v-model="showSettings" title="品牌设置" width="90%" style="max-width: 480px" :close-on-click-modal="false">
      <!-- 品牌列表 -->
      <div class="setting-brand-tabs">
        <div
          v-for="(brand, index) in settings.brands"
          :key="brand.id"
          class="setting-brand-tab"
          :class="{ active: index === editingBrandIndex }"
          @click="switchEditBrand(index)"
        >
          {{ brand.name }}
          <el-icon
            v-if="settings.brands.length > 1"
            class="brand-remove-icon"
            @click.stop="removeBrand(index)"
          ><Close /></el-icon>
        </div>
        <div class="setting-brand-tab add-btn" @click="addBrand">
          <el-icon><Plus /></el-icon>
        </div>
      </div>

      <!-- 当前品牌配置 -->
      <el-form label-width="80px" label-position="top" class="brand-form">
        <el-form-item label="品牌名称">
          <el-input v-model="editingBrand.name" placeholder="输入品牌名" />
        </el-form-item>
        <el-form-item :label="editingBrand.useTieredRate ? '默认收款比例（未匹配区间时使用）' : '收款比例'">
          <el-input-number
            v-model="editingBrand.receivedRate"
            :min="0" :max="999" :precision="2" :controls="false"
            placeholder="如 65"
          />
          <span class="setting-unit">%</span>
        </el-form-item>
        <el-form-item label="阶梯折扣">
          <div class="tier-config">
            <el-switch
              v-model="editingBrand.useTieredRate"
              active-text="启用"
              inactive-text="关闭"
              style="margin-bottom: 10px"
            />
            <template v-if="editingBrand.useTieredRate">
              <div
                v-for="(tier, tIdx) in (editingBrand.tiers || [])"
                :key="tIdx"
                class="tier-row"
              >
                <el-input-number
                  v-model="tier.minAmount"
                  :min="0" :precision="0" :controls="false"
                  placeholder="最低"
                  size="small"
                  class="tier-input"
                />
                <span class="tier-sep">~</span>
                <el-input-number
                  v-model="tier.maxAmount"
                  :min="0" :precision="0" :controls="false"
                  placeholder="0=无上限"
                  size="small"
                  class="tier-input"
                />
                <span class="tier-sep">元</span>
                <el-input-number
                  v-model="tier.receivedRate"
                  :min="0" :max="999" :precision="2" :controls="false"
                  placeholder="折扣"
                  size="small"
                  class="tier-input-rate"
                />
                <span class="tier-sep">%</span>
                <el-button
                  :icon="Delete"
                  type="danger"
                  text
                  size="small"
                  @click="removeTier(editingBrandIndex, tIdx)"
                />
              </div>
              <el-button
                type="primary"
                text
                size="small"
                :icon="Plus"
                @click="addTier(editingBrandIndex)"
                style="margin-top: 4px"
              >添加区间</el-button>
              <div class="tier-hint">最高金额填 0 表示无上限，如：201 ~ 0 元表示 201 元以上</div>
            </template>
          </div>
        </el-form-item>
        <el-form-item label="支出比例">
          <el-input-number
            v-model="editingBrand.expenseRate"
            :min="0" :max="999" :precision="2" :controls="false"
            placeholder="如 60"
          />
          <span class="setting-unit">%</span>
        </el-form-item>
        <el-form-item label="手续费率">
          <el-input-number
            v-model="editingBrand.feeRate"
            :min="0" :max="100" :precision="1" :step="0.1" :controls="false"
          />
          <span class="setting-unit">%</span>
        </el-form-item>
        <el-form-item label="抹零方式">
          <el-radio-group v-model="editingBrand.roundMode">
            <el-radio value="none">不抹零</el-radio>
            <el-radio value="fen">去分</el-radio>
            <el-radio value="mao">去毛</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showSettings = false">取消</el-button>
        <el-button type="primary" @click="handleSettingsSave">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --bg-page: #f5f7fa;
  --bg-card: #fff;
  --bg-card-hover: #f5f7fa;
  --text-primary: #303133;
  --text-regular: #606266;
  --text-secondary: #909399;
  --text-placeholder: #c0c4cc;
  --border-light: #e4e7ed;
  --border-lighter: #f2f3f5;
  --border-extra-light: #fafafa;
  --shadow-card: 0 2px 12px rgba(0, 0, 0, 0.06);
  --brand-tab-bg: #fff;
  --discount-bg: #fdf6ec;
  --discount-color: #e6a23c;
  --history-brand-bg: #ecf5ff;
  --setting-tab-bg: #f5f7fa;
  --setting-border: #f2f3f5;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg-page: #1a1a1a;
    --bg-card: #2b2b2b;
    --bg-card-hover: #333;
    --text-primary: #e0e0e0;
    --text-regular: #b0b0b0;
    --text-secondary: #888;
    --text-placeholder: #555;
    --border-light: #444;
    --border-lighter: #383838;
    --border-extra-light: #333;
    --shadow-card: 0 2px 12px rgba(0, 0, 0, 0.3);
    --brand-tab-bg: #2b2b2b;
    --discount-bg: #3d3520;
    --discount-color: #e6a23c;
    --history-brand-bg: #1e3a5f;
    --setting-tab-bg: #333;
    --setting-border: #444;
  }

  body .el-dialog {
    --el-dialog-bg-color: #2b2b2b;
    --el-dialog-title-font-size: 16px;
  }
  body .el-form-item__label {
    color: var(--text-regular) !important;
  }
  body .el-input__wrapper,
  body .el-input-number {
    --el-input-bg-color: #333;
    --el-input-border-color: #444;
    --el-input-text-color: #e0e0e0;
  }
  body .el-radio-button__inner {
    background: #333;
    color: #b0b0b0;
    border-color: #444;
  }
  body .el-radio__label {
    color: #b0b0b0;
  }
}

body {
  font-family: "PingFang SC", "Microsoft YaHei", -apple-system, sans-serif;
  background: var(--bg-page);
  color: var(--text-primary);
  user-select: none;
  overflow-y: auto;
}
</style>

<style scoped>
.app-container {
  max-width: 560px;
  margin: 0 auto;
  padding: 20px 24px 12px;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.header-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.input-section {
  background: var(--bg-card);
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow-card);
}

.input-row {
  display: flex;
  align-items: center;
  margin-bottom: 16px;
  gap: 10px;
}

.input-row:last-child {
  margin-bottom: 0;
}

.input-row label {
  width: 72px;
  font-size: 14px;
  color: var(--text-regular);
  flex-shrink: 0;
  text-align: right;
}

.input-number {
  flex: 1;
}

.brand-info-bar {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 14px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-lighter);
}

.brand-info-item {
  font-size: 12px;
  color: var(--text-secondary);
}

.brand-info-item b {
  color: var(--text-primary);
  font-weight: 600;
}

.no-config-tip {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 24px 0;
}

.no-config-tip p {
  font-size: 14px;
  color: var(--text-secondary);
}

.unit {
  font-size: 14px;
  color: var(--text-secondary);
  width: 20px;
  flex-shrink: 0;
}

.round-mode-group :deep(.el-radio-button__inner) {
  padding: 7px 14px;
  font-size: 13px;
}

.discount-label {
  font-size: 13px;
  color: var(--discount-color);
  background: var(--discount-bg);
  padding: 2px 8px;
  border-radius: 4px;
  white-space: nowrap;
}

.result-section {
  background: #2b2b2b;
  border-radius: 8px;
  padding: 16px 20px;
  margin-top: 16px;
  font-family: "Menlo", "Consolas", "Courier New", monospace;
  font-size: 14px;
  line-height: 1.8;
}

.result-empty {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 80px;
  color: #6a9955;
  font-size: 14px;
}

.result-empty::before {
  content: '// ';
}

.result-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2px 0;
  border-bottom: none;
}

.result-item:last-of-type {
  border-bottom: none;
}

.result-label {
  font-size: 14px;
  color: #6a9955;
}

.result-label::before {
  content: '// ';
}

.result-value {
  font-size: 14px;
  font-weight: 400;
  font-family: inherit;
  color: #d4d4d4;
}

.result-profit {
  padding: 6px 0 2px;
  margin-top: 4px;
  border-top: none;
  border-bottom: none;
}

.result-profit .result-label {
  color: #569cd6;
}

.result-profit .result-label::before {
  content: '=> ';
  color: #569cd6;
}

.profit-value {
  font-size: 18px;
  font-weight: 700;
}

.quote-section {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid #3e3e3e;
}

.quote-text {
  flex: 1;
  font-size: 13px;
  color: #808080;
  line-height: 1.4;
  font-family: inherit;
}

.history-section {
  background: var(--bg-card);
  border-radius: 12px;
  padding: 16px 20px;
  margin-top: 16px;
  box-shadow: var(--shadow-card);
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  margin-bottom: 8px;
}

.history-title {
  font-size: 13px;
  color: var(--text-secondary);
}

.history-toggle {
  font-size: 12px;
  color: #409eff;
  margin-left: 6px;
}

.history-more {
  text-align: center;
  padding: 8px 0 2px;
  font-size: 12px;
  color: #409eff;
  cursor: pointer;
}

.history-more:hover {
  color: #337ecc;
}

.history-list {
  max-height: 400px;
  overflow-y: auto;
}

.history-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
  font-size: 13px;
  border-bottom: 1px solid var(--border-extra-light);
}

.history-item:last-child {
  border-bottom: none;
}

.history-main {
  color: var(--text-regular);
}

.history-time {
  color: var(--text-placeholder);
  font-size: 12px;
  flex-shrink: 0;
  margin-left: 8px;
}

.footer {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 16px;
  margin-top: auto;
  padding: 16px 0 8px;
  font-size: 12px;
  color: var(--text-placeholder);
}

.setting-unit {
  margin-left: 8px;
  font-size: 14px;
  color: var(--text-secondary);
}

.setting-hint {
  margin-left: 8px;
  font-size: 12px;
  color: var(--text-placeholder);
}

.brand-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.brand-tab {
  padding: 6px 16px;
  border-radius: 20px;
  font-size: 13px;
  color: var(--text-regular);
  background: var(--brand-tab-bg);
  border: 1px solid var(--border-light);
  cursor: pointer;
  transition: all 0.2s;
}

.brand-tab:hover {
  border-color: #409eff;
  color: #409eff;
}

.brand-tab.active {
  background: #409eff;
  color: #fff;
  border-color: #409eff;
}

.setting-brand-tabs {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--setting-border);
}

.setting-brand-tab {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 12px;
  border-radius: 4px;
  font-size: 13px;
  color: var(--text-regular);
  background: var(--setting-tab-bg);
  cursor: pointer;
  transition: all 0.2s;
}

.setting-brand-tab:hover {
  background: #ecf5ff;
  color: #409eff;
}

.setting-brand-tab.active {
  background: #409eff;
  color: #fff;
}

.setting-brand-tab.add-btn {
  background: transparent;
  border: 1px dashed var(--border-light);
  color: var(--text-secondary);
}

.setting-brand-tab.add-btn:hover {
  border-color: #409eff;
  color: #409eff;
}

.brand-remove-icon {
  font-size: 12px;
  opacity: 0.6;
  margin-left: 2px;
}

.brand-remove-icon:hover {
  opacity: 1;
}

.history-brand {
  display: inline-block;
  font-size: 11px;
  background: var(--history-brand-bg);
  color: #409eff;
  padding: 0 6px;
  border-radius: 3px;
  margin-right: 4px;
}

.brand-form :deep(.el-form-item) {
  margin-bottom: 12px;
}

.brand-form :deep(.el-form-item__label) {
  padding-bottom: 4px;
  font-size: 13px;
  color: var(--text-regular);
}

/* ──────────── 阶梯折扣 ──────────── */
.tier-matched {
  color: #409eff !important;
  font-weight: 500;
}

.tier-config {
  width: 100%;
}

.tier-row {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 8px;
}

.tier-input {
  width: 72px !important;
  flex-shrink: 0;
}

.tier-input-rate {
  width: 68px !important;
  flex-shrink: 0;
}

.tier-sep {
  font-size: 12px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.tier-hint {
  font-size: 11px;
  color: var(--text-placeholder);
  margin-top: 6px;
  line-height: 1.5;
}

/* ──────────── 吸附面板 ──────────── */
.snap-panel {
  padding: 4px 0;
}

.snap-panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-lighter);
}

.snap-panel-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.snap-label {
  font-size: 13px;
  color: var(--text-secondary);
  width: 60px;
  flex-shrink: 0;
  text-align: right;
}

.snap-value {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
}

.snap-title-ellipsis {
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 400;
  font-size: 12px;
}

.snap-unit {
  font-size: 12px;
  color: var(--text-placeholder);
}

.snap-status {
  font-size: 13px;
  font-weight: 500;
}

.snap-status-ok {
  color: #67c23a;
}

.snap-status-searching {
  color: #e6a23c;
}

.snap-status-off {
  color: var(--text-placeholder);
}

.snap-panel-actions {
  margin-top: 14px;
  padding-top: 10px;
  border-top: 1px solid var(--border-lighter);
}

@media (max-width: 500px) {
  .app-container {
    padding: 12px 12px 8px;
  }

  .header-title {
    font-size: 16px;
  }

  .brand-tabs {
    gap: 6px;
    margin-bottom: 12px;
  }

  .brand-tab {
    padding: 4px 12px;
    font-size: 12px;
  }

  .input-section {
    padding: 14px;
  }

  .input-row {
    gap: 6px;
    margin-bottom: 12px;
  }

  .input-row label {
    width: 60px;
    font-size: 13px;
  }

  .result-section {
    padding: 12px 14px;
    font-size: 13px;
  }

  .setting-brand-tabs {
    gap: 6px;
  }

  .setting-brand-tab {
    padding: 4px 10px;
    font-size: 12px;
  }
}
</style>