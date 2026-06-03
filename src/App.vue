<template>
  <div class="app">
    <div class="header">
      <h1>Vision Agent</h1>
      <p class="subtitle">Tell me what to do on your computer</p>
    </div>

    <div class="input-area">
      <input
        v-model="prompt"
        class="prompt-input"
        placeholder='e.g. "Open Spotify and search Flashing Lights"'
        :disabled="loading"
        @keyup.enter="runPrompt"
      />
      <button class="run-btn" :disabled="loading || !prompt.trim()" @click="runPrompt">
        {{ loading ? 'Running...' : 'Run' }}
      </button>
    </div>

    <div v-if="steps.length" class="log">
      <div class="log-header">Action Log</div>
      <div
        v-for="(step, i) in steps"
        :key="i"
        class="log-item"
      >
        {{ step }}
      </div>
    </div>

    <div v-if="error" class="error-box">
      ⚠️ {{ error }}
    </div>

    <div v-if="!loading && !steps.length && !error" class="hint">
      The agent will open apps, find UI elements, and interact with them automatically.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const prompt = ref('')
const steps = ref<string[]>([])
const loading = ref(false)
const error = ref('')

async function runPrompt() {
  if (!prompt.value.trim() || loading.value) return

  steps.value = []
  error.value = ''
  loading.value = true

  try {
    const results = await invoke<string[]>('run_prompt', {
      prompt: prompt.value
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
}

.prompt-input {
  flex: 1;
  padding: 12px 16px;
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 8px;
  color: #f0f0f0;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.prompt-input:focus {
  border-color: #555;
}

.prompt-input:disabled {
  opacity: 0.5;
}

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
}

.run-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
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
}

.log-item {
  padding: 10px 16px;
  font-size: 13px;
  color: #ccc;
  border-bottom: 1px solid #1f1f1f;
  font-family: monospace;
}

.log-item:last-child {
  border-bottom: none;
}

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
</style>