export type RoundMode = 'none' | 'fen' | 'mao'

export interface CalcInput {
  originalAmount: number
  receivedRate: number
  roundMode: RoundMode
  feeRate: number
  expenseRate: number
}

export interface CalcResult {
  receivedAmount: number
  expenseAmount: number
  serviceFee: number
  netProfit: number
}

export interface HistoryRecord {
  id: number
  brandName: string
  originalAmount: number
  receivedRate: number
  expenseRate: number
  roundMode: RoundMode
  feeRate: number
  receivedAmount: number
  expenseAmount: number
  serviceFee: number
  netProfit: number
  time: string
}

/** 抹零：去分=保留到角，去毛=保留到元 */
export function roundDown(amount: number, mode: RoundMode): number {
  if (mode === 'fen') return Math.floor(amount * 10) / 10
  if (mode === 'mao') return Math.floor(amount)
  return amount
}

export function calculate(input: CalcInput): CalcResult {
  const raw = +(input.originalAmount * input.receivedRate / 100).toFixed(2)
  const receivedAmount = roundDown(raw, input.roundMode)
  const expenseAmount = +(input.originalAmount * input.expenseRate / 100).toFixed(2)
  const serviceFee = Math.ceil(receivedAmount * input.feeRate * 100) / 100
  const netProfit = +(receivedAmount - expenseAmount - serviceFee).toFixed(2)
  return { receivedAmount, expenseAmount, serviceFee, netProfit }
}

/** 生成复制报价文案 */
export function generateQuote(originalAmount: number, receivedAmount: number): string {
  return `原价${originalAmount}折后价格${receivedAmount}哦亲！`
}

const HISTORY_KEY = 'catering-calc-history'
const MAX_HISTORY = 100

export function getHistory(): HistoryRecord[] {
  try {
    const data = localStorage.getItem(HISTORY_KEY)
    return data ? JSON.parse(data) : []
  } catch {
    return []
  }
}

export function saveHistory(record: Omit<HistoryRecord, 'id' | 'time'>): HistoryRecord[] {
  const list = getHistory()
  const newRecord: HistoryRecord = {
    ...record,
    id: Date.now(),
    time: new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  }
  list.unshift(newRecord)
  if (list.length > MAX_HISTORY) list.length = MAX_HISTORY
  localStorage.setItem(HISTORY_KEY, JSON.stringify(list))
  return list
}

export function clearHistory(): HistoryRecord[] {
  localStorage.removeItem(HISTORY_KEY)
  return []
}

const SETTINGS_KEY = 'catering-calc-settings'

export interface Brand {
  id: string
  name: string
  receivedRate: number
  expenseRate: number
  feeRate: number
  roundMode: RoundMode
}

export interface Settings {
  activeBrandId: string
  brands: Brand[]
}

function genId(): string {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 6)
}

const DEFAULT_BRAND: Brand = {
  id: 'default',
  name: '默认品牌',
  receivedRate: 65,
  expenseRate: 0,
  feeRate: 1.6,
  roundMode: 'none',
}

const DEFAULT_SETTINGS: Settings = {
  activeBrandId: 'default',
  brands: [{ ...DEFAULT_BRAND }],
}

export function getSettings(): Settings {
  try {
    const data = localStorage.getItem(SETTINGS_KEY)
    if (!data) return { ...DEFAULT_SETTINGS, brands: [{ ...DEFAULT_BRAND }] }
    const parsed = JSON.parse(data)
    if (parsed.brands && parsed.brands.length > 0) {
      return parsed as Settings
    }
    return { ...DEFAULT_SETTINGS, brands: [{ ...DEFAULT_BRAND }] }
  } catch {
    return { ...DEFAULT_SETTINGS, brands: [{ ...DEFAULT_BRAND }] }
  }
}

export function saveSettings(settings: Settings): void {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings))
}

export function getActiveBrand(settings: Settings): Brand {
  return settings.brands.find(b => b.id === settings.activeBrandId) || settings.brands[0] || { ...DEFAULT_BRAND }
}

export function createBrand(name: string): Brand {
  return {
    id: genId(),
    name,
    receivedRate: 65,
    expenseRate: 0,
    feeRate: 1.6,
    roundMode: 'none',
  }
}
