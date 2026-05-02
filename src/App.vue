<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import AppLayout from "@components/layout/AppLayout.vue";
import SplashScreen from "@components/splash/SplashScreen.vue";
import UpdateModal from "@components/common/UpdateModal.vue";
import TermsDialog from "@components/common/TermsDialog.vue";
import ToastContainer from "@components/common/ToastContainer.vue";
import { useUpdateStore } from "@stores/updateStore";
import { useSettingsStore } from "@stores/settingsStore";
import { useServerStore } from "@stores/serverStore";
import { useGlobalMessage } from "@composables/useMessage";
import { isBrowserEnv } from "@api/tauri";
import {
  applyTheme,
  applyFontSize,
  applyFontFamily,
  applyMinimalMode,
  applyDeveloperMode,
} from "@utils/theme";
import { SETTINGS_UPDATE_EVENT, type SettingsUpdateEvent } from "@stores/settingsStore";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

function playNotificationSound() {
  try {
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);

    oscillator.type = "sine";
    oscillator.frequency.setValueAtTime(880, audioContext.currentTime);
    oscillator.frequency.setValueAtTime(1100, audioContext.currentTime + 0.1);

    gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.3);

    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.3);
  } catch (e) {
    console.warn("播放提示音失败:", e);
  }
}

const showSplash = ref(true);
const isInitializing = ref(true);
const showTermsDialog = ref(false);
const updateStore = useUpdateStore();
const settingsStore = useSettingsStore();
const serverStore = useServerStore();
const globalMessage = useGlobalMessage();

interface ServerStartFallbackEventPayload {
  serverId: string;
  serverName: string;
  fromMode: string;
  toMode: string;
  reason: string;
}

let serverErrorUnlisten: UnlistenFn | null = null;
let serverStartFallbackUnlisten: UnlistenFn | null = null;

onMounted(async () => {
  if (!isBrowserEnv()) {
    serverErrorUnlisten = await listen("server-error", () => {
      playNotificationSound();
    });
    serverStartFallbackUnlisten = await listen<ServerStartFallbackEventPayload>(
      "server-start-fallback",
      ({ payload }) => {
        const displayName = payload.serverName || payload.serverId;
        globalMessage.warning(
          `Server ${displayName} failed to start via JAR, automatically fell back to ${payload.toMode} mode (${payload.reason})`,
          5000,
        );
      },
    );
  }

  await new Promise((resolve) => setTimeout(resolve, 500));

  window.addEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdate as EventListener);

  try {
    await settingsStore.loadSettings();
    const settings = settingsStore.settings;
    applyTheme(settings.theme || "auto");
    applyFontSize(settings.font_size || 14);
    applyFontFamily(settings.font_family || "");
    applyMinimalMode(settings.minimal_mode || false);
    applyDeveloperMode(settings.developer_mode || false);

    try {
      await serverStore.refreshList();
    } catch (serverErr) {
      console.warn("Failed to load servers during startup:", serverErr);
    }
  } catch (e) {
    console.error("Failed to load settings during startup:", e);
  } finally {
    isInitializing.value = false;
  }
});

onUnmounted(() => {
  if (serverErrorUnlisten) {
    serverErrorUnlisten();
    serverErrorUnlisten = null;
  }
  if (serverStartFallbackUnlisten) {
    serverStartFallbackUnlisten();
    serverStartFallbackUnlisten = null;
  }

  window.removeEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdate as EventListener);
});

async function handleAgreeTerms() {
  try {
    await settingsStore.updatePartial({ agreed_to_terms: true });
    showTermsDialog.value = false;
  } catch (error) {
    console.error("Failed to save terms agreement:", error);
  }
}

function handleSplashReady() {
  if (isInitializing.value) return;
  showSplash.value = false;

  const checkTerms = () => {
    if (settingsStore.isLoaded) {
      const settings = settingsStore.settings;
      if (!settings.agreed_to_terms) {
        showTermsDialog.value = true;
      }
      if (!import.meta.env.DEV) {
        updateStore.checkForUpdateOnStartup();
      }
    } else {
      setTimeout(checkTerms, 50);
    }
  };

  checkTerms();
}

function handleUpdateModalClose() {
  updateStore.hideUpdateModal();
}

function handleSettingsUpdate(e: CustomEvent<SettingsUpdateEvent>) {
  const { settings } = e.detail;
  applyDeveloperMode(settings.developer_mode || false);
}
</script>

<template>
  <transition name="splash-fade">
    <SplashScreen v-if="showSplash" :loading="isInitializing" @ready="handleSplashReady" />
  </transition>

  <template v-if="!showSplash">
    <AppLayout />

    <UpdateModal
      v-if="updateStore.isUpdateModalVisible && updateStore.isUpdateAvailable"
      @close="handleUpdateModalClose"
    />

    <TermsDialog
      :visible="showTermsDialog"
      @agree="handleAgreeTerms"
      @close="showTermsDialog = false"
    />

    <ToastContainer />
  </template>
</template>

<style src="@styles/app.css"></style>
