import { reactive } from 'vue'

export interface DialogOptions {
  title: string
  message: string
  showInput?: boolean
  inputPlaceholder?: string
  inputType?: string
  confirmText?: string
  cancelText?: string
}

const state = reactive({
  visible: false,
  opts: { title: '', message: '' } as DialogOptions,
  inputValue: '',
})

let resolve: ((v: boolean | string | null) => void) | null = null

export function showConfirm(opts: DialogOptions): Promise<boolean> {
  state.opts = { ...opts, showInput: false }
  state.inputValue = ''
  state.visible = true
  return new Promise(r => { resolve = r as any })
}

export function showPrompt(opts: DialogOptions): Promise<string | null> {
  state.opts = { ...opts, showInput: true, inputType: opts.inputType || 'text' }
  state.inputValue = ''
  state.visible = true
  return new Promise(r => { resolve = r as any })
}

export function confirmDialog() {
  if (state.opts.showInput) {
    resolve?.(state.inputValue)
  } else {
    resolve?.(true)
  }
  state.visible = false
  resolve = null
}

export function cancelDialog() {
  resolve?.(state.opts.showInput ? null : false)
  state.visible = false
  resolve = null
}

export function dialogState() {
  return state
}
