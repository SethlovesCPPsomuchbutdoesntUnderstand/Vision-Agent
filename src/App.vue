Set-Content "C:\Users\ADMIN\vision-agent\src\App.vue" @'
<template>
  <div class="app">
    <div class="header">
      <h1>Vision Agent</h1>
      <p class="subtitle">Tell me what to do on your computer</p>
    </div>

    <div class="input-area">
      <div class="input-wrapper">
        <input
          v-model="prompt"
          class="prompt-input"
          placeholder="e.g. Open Spotify and search Flashing Lights"
          :disabled="loading"
          :class="{ blocked: isBlocked }"
          maxlength="500"
          @keyup.enter="runPrompt"
          @input="checkPrompt"
        />
        <span class="char-count" :class="{ warn: prompt.length > 400 }">
          {{ prompt.length }}/500
        </span>
      </div>
      <button class="run-btn" :disabled="loading || !prompt.trim() || isBlocked" @click="runPrompt">
        {{ loading ? 'Running...' : 'Run' }}
      </button>
    </div>

    <div v-if="isBlocked" class="blocked-warning">
      ⛔ Prompt contains restricted content and cannot be run.
    </div>

    <div v-if="steps.length" class="log">
      <div class="log-header">
        Action Log
        <span class="log-count">{{ steps.length }} steps</span>
      </div>
      <div
        v-for="(step, i) in steps"
        :key="i"
        class="log-item"
        :class="getStepClass(step)"
      >
        {{ step }}
      </div>
    </div>

    <div v-if="error" class="error-box">
      {{ error }}
    </div>

    <div v-if="!loading && !steps.length && !error" class="hint">
      The agent will open apps and interact with them automatically.
      <div class="security-note">
        🔒 System commands, file deletion, and shell access are blocked.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const prompt = ref('')
const steps = ref<string[]>([])
const loading = ref(false)
const error = ref('')

const BLOCKED_KEYWORDS = [
  'delete', 'format', 'rmdir', 'rm -rf', 'del ',
  'cmd', 'powershell', 'registry', 'regedit',
  'shutdown', 'restart', 'taskkill', 'net user',
  'password', 'credential', 'ignore previous',
  'ignore instructions', 'jailbreak', 'act as',
  'you are now', 'disregard', 'system prompt'
]

const isBlocked = computed(() => {
  const lower = prompt.value.toLowerCase()
  return BLOCKED_KEYWORDS.some(k => lower.includes(k))
})

function checkPrompt() {
  error.value = ''
}

function getStepClass(step: string) {
  if (step.startsWith('✅')) return 'step-done'
  if (step.startsWith('⛔')) return 'step-blocked'
  if (step.startsWith('📸')) return 'step-screenshot'
  return ''
}

async function runPrompt() {
  if (!prompt.value.trim() || loading.value || isBlocked.value) return

  steps.value = []
  error.value = ''
  loading.value = true

  try {
    const results = await invoke<string[]>('run_prompt', {
      prompt: prompt.value.trim()
    })
    steps.value = results
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}
</script>

<style>
* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  font-family: system-ui, sans-serif;
  background: #0f0f0f;
  color: #f0f0f0;
  height: 100vh;
}

.app {
  max-width: 680px;
  margin: 0 auto;
  padding: 48px 24px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.header h1 {
  font-size: 28px;
  font-weight: 600;
  margin-bottom: 6px;
}

.subtitle {
  color: #888;
  font-size: 14px;
}

.input-area {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.input-wrapper {
  flex: 1;
  position: relative;
}

.prompt-input {
  width: 100%;
  padding: 12px 16px;
  padding-right: 56px;
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 8px;
  color: #f0f0f0;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.prompt-input:focus { border-color: #555; }
.prompt-input:disabled { opacity: 0.5; }
.prompt-input.blocked { border-color: #5a2020; }

.char-count {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 11px;
  color: #555;
  pointer-events: none;
}

.char-count.warn { color: #cc8800; }

.run-btn {
  padding: 12px 24px;
  background: #f0f0f0;
  color: #0f0f0f;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.2s;
  white-space: nowrap;
}

.run-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.blocked-warning {
  padding: 10px 14px;
  background: #2a1010;
  border: 1px solid #5a2020;
  border-radius: 8px;
  font-size: 13px;
  color: #ff8080;
}

.log {
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-radius: 8px;
  overflow: hidden;
}

.log-header {
  padding: 10px 16px;
  font-size: 12px;
  font-weight: 600;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border-bottom: 1px solid #2a2a2a;
  display: flex;
  justify-content: space-between;
}

.log-count {
  font-weight: 400;
  color: #555;
}

.log-item {
  padding: 10px 16px;
  font-size: 13px;
  color: #ccc;
  border-bottom: 1px solid #1f1f1f;
  font-family: monospace;
}

.log-item:last-child { border-bottom: none; }
.log-item.step-done { color: #6dbb6d; }
.log-item.step-blocked { color: #ff8080; }
.log-item.step-screenshot { color: #88aaff; }

.error-box {
  padding: 14px 16px;
  background: #2a1010;
  border: 1px solid #5a2020;
  border-radius: 8px;
  font-size: 13px;
  color: #ff8080;
}

.hint {
  color: #555;
  font-size: 13px;
  text-align: center;
  padding-top: 12px;
}

.security-note {
  margin-top: 8px;
  font-size: 12px;
  color: #444;
}
</style>
'@