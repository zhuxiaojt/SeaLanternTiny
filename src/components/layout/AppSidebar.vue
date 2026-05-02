<script setup lang="ts">
import { computed, ref, nextTick, watch, onMounted, onUnmounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useUiStore } from "@stores/uiStore";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";
import SLSelect from "@components/common/SLSelect.vue";
import {
  Home,
  Plus,
  Terminal,
  Settings,
  Users,
  Sliders,
  PaintRoller,
  Info,
  Server,
  ChevronLeft,
  DownloadIcon,
  type LucideIcon,
} from "lucide-vue-next";
import logoSvg from "@assets/logo.svg";
import { isMacOSPlatform } from "@utils/platform";

const iconMap: Record<string, LucideIcon> = {
  home: Home,
  plus: Plus,
  terminal: Terminal,
  settings: Settings,
  users: Users,
  sliders: Sliders,
  paint: PaintRoller,
  info: Info,
  server: Server,
  download: DownloadIcon,
};

function getNavIcon(name: string): LucideIcon {
  return iconMap[name] ?? Info;
}

const router = useRouter();
const route = useRoute();
const ui = useUiStore();
const serverStore = useServerStore();
const navIndicator = ref<HTMLElement | null>(null);
const sidebarTransitioning = ref(false);
const isMacOS = isMacOSPlatform();
let indicatorSyncInterval: ReturnType<typeof setInterval> | null = null;
let indicatorSyncTimeout: ReturnType<typeof setTimeout> | null = null;

interface NavItem {
  name: string;
  path: string;
  icon: string;
  labelKey: string;
  label: string;
  group: string;
  after?: string;
  children?: NavItem[];
}

const staticNavItems: NavItem[] = [
  {
    name: "home",
    path: "/",
    icon: "home",
    labelKey: "common.home",
    label: i18n.t("common.home"),
    group: "main",
  },
  {
    name: "create",
    path: "/create",
    icon: "plus",
    labelKey: "common.create_server",
    label: i18n.t("common.create_server"),
    group: "main",
  },
  {
    name: "download",
    path: "/download",
    icon: "download",
    labelKey: "common.download",
    label: i18n.t("common.download"),
    group: "main",
  },
  {
    name: "console",
    path: "/console",
    icon: "terminal",
    labelKey: "common.console",
    label: i18n.t("common.console"),
    group: "server",
  },
  {
    name: "config",
    path: "/config",
    icon: "sliders",
    labelKey: "common.config_edit",
    label: i18n.t("common.config_edit"),
    group: "server",
  },
  {
    name: "players",
    path: "/players",
    icon: "users",
    labelKey: "common.player_manage",
    label: i18n.t("common.player_manage"),
    group: "server",
  },
  {
    name: "paint",
    path: "/paint",
    icon: "paint",
    labelKey: "common.personalize",
    label: i18n.t("common.personalize"),
    group: "system",
  },
  {
    name: "settings",
    path: "/settings",
    icon: "settings",
    labelKey: "common.settings",
    label: i18n.t("common.settings"),
    group: "system",
  },
];

const navItems = computed<NavItem[]>(() => {
  return staticNavItems;
});

function navigateTo(path: string) {
  router.push(path);
}

function updateNavIndicator() {
  nextTick(() => {
    if (!navIndicator.value) return;

    const activeNavItem = document.querySelector(".nav-item.active");
    const sidebarNav = document.querySelector(".sidebar-nav");

    if (activeNavItem && sidebarNav && navIndicator.value.parentElement) {
      const navItemRect = activeNavItem.getBoundingClientRect();
      const sidebarNavRect = sidebarNav.getBoundingClientRect();

      const top =
        navItemRect.top - sidebarNavRect.top + sidebarNav.scrollTop + (navItemRect.height - 16) / 2;

      navIndicator.value.style.display = "block";

      void navIndicator.value.offsetHeight;

      requestAnimationFrame(() => {
        navIndicator.value!.style.top = `${top}px`;
      });
    }
  });
}

function startIndicatorSyncDuringSidebarTransition() {
  if (indicatorSyncInterval) {
    clearInterval(indicatorSyncInterval);
    indicatorSyncInterval = null;
  }
  if (indicatorSyncTimeout) {
    clearTimeout(indicatorSyncTimeout);
    indicatorSyncTimeout = null;
  }

  sidebarTransitioning.value = true;
  indicatorSyncInterval = setInterval(() => {
    updateNavIndicator();
  }, 16);

  indicatorSyncTimeout = setTimeout(() => {
    if (indicatorSyncInterval) {
      clearInterval(indicatorSyncInterval);
      indicatorSyncInterval = null;
    }
    sidebarTransitioning.value = false;
    updateNavIndicator();
  }, 360);
}

watch(
  () => ui.sidebarCollapsed,
  () => {
    updateNavIndicator();
    startIndicatorSyncDuringSidebarTransition();
  },
);

watch(
  () => route.path,
  () => {
    nextTick(() => {
      updateNavIndicator();
    });
  },
);

onMounted(async () => {
  await serverStore.refreshList();
  nextTick(() => {
    updateNavIndicator();
  });
});

function handleServerChange(value: string) {
  serverStore.setCurrentServer(value);
  if (
    route.path.startsWith("/console") ||
    route.path.startsWith("/config") ||
    route.path.startsWith("/players")
  ) {
    const currentPath = route.path.split("/")[1];
    router.push(`/${currentPath}/${value}`);
  }
}

const serverOptions = computed(() => {
  return serverStore.servers.map((s) => ({
    label: s.name,
    value: s.id,
  }));
});

const currentServerRef = computed({
  get: () => serverStore.currentServerId ?? undefined,
  set: (v) => {
    if (v) handleServerChange(v);
  },
});

watch(
  () => serverOptions.value.length,
  () => {
    updateNavIndicator();
  },
);

onMounted(() => {
  window.addEventListener("resize", updateNavIndicator);

  const sidebarNav = document.querySelector(".sidebar-nav");
  if (sidebarNav) {
    sidebarNav.addEventListener("scroll", updateNavIndicator);
  }
});

onUnmounted(() => {
  if (indicatorSyncInterval) {
    clearInterval(indicatorSyncInterval);
    indicatorSyncInterval = null;
  }
  if (indicatorSyncTimeout) {
    clearTimeout(indicatorSyncTimeout);
    indicatorSyncTimeout = null;
  }

  window.removeEventListener("resize", updateNavIndicator);

  const sidebarNav = document.querySelector(".sidebar-nav");
  if (sidebarNav) {
    sidebarNav.removeEventListener("scroll", updateNavIndicator);
  }
});

function isActive(path: string): boolean {
  if (path === "/") return route.path === "/";
  return route.path.startsWith(path);
}

interface NavGroup {
  group: string;
  items: NavItem[];
}

const orderedNavGroups = computed<NavGroup[]>(() => {
  const groups: NavGroup[] = [];
  let currentGroup: NavGroup | null = null;

  for (const item of navItems.value) {
    if (!currentGroup || currentGroup.group !== item.group) {
      currentGroup = { group: item.group, items: [] };
      groups.push(currentGroup);
    }
    currentGroup.items.push(item);
  }

  return groups;
});

function getAppName() {
  const now = new Date();
  if (now.getMonth() == 3 && now.getDate() == 1) {
    return i18n.t("common.easter_name");
  }
  return i18n.t("common.app_name");
}
</script>

<template>
  <aside
    class="sidebar glass-strong"
    :class="{
      collapsed: ui.sidebarCollapsed,
      'macos-overlay': isMacOS,
      'sidebar-transitioning': sidebarTransitioning,
    }"
  >
    <div class="sidebar-logo" @click="navigateTo('/')">
      <div class="logo-icon">
        <img :src="logoSvg" width="28" height="28" :alt="i18n.t('common.app_name')" />
      </div>
      <transition name="fade">
        <span v-if="!ui.sidebarCollapsed" class="logo-text">{{ getAppName() }}</span>
      </transition>
    </div>
    <nav class="sidebar-nav">
      <div class="nav-active-indicator" ref="navIndicator"></div>
      <SLSelect
        v-if="serverOptions.length > 0"
        v-model="currentServerRef"
        :options="serverOptions"
        :collapsed="ui.sidebarCollapsed"
        :icon="Server"
        :placeholder="i18n.t('common.select_server')"
        variant="server"
        dropdown-align="right"
        class="server-selector"
      />

      <template v-for="(group, gi) in orderedNavGroups" :key="gi">
        <div v-if="group.group !== 'server' || serverOptions.length > 0" class="nav-group">
          <div v-for="item in group.items" :key="item.name">
            <div
              class="nav-item"
              :class="{ active: isActive(item.path) }"
              @click="navigateTo(item.path)"
              :title="ui.sidebarCollapsed ? item.label : ''"
            >
              <component
                :is="getNavIcon(item.icon)"
                class="nav-icon"
                :size="20"
                :stroke-width="1.8"
              />
              <transition name="fade">
                <span v-if="!ui.sidebarCollapsed" class="nav-label">
                  {{ item.labelKey ? i18n.t(item.labelKey) : item.label }}
                </span>
              </transition>
            </div>
          </div>
        </div>
      </template>

      <div class="nav-group lower-side">
        <div
          class="nav-item"
          :class="{ active: isActive('/about') }"
          @click="navigateTo('/about')"
          :title="ui.sidebarCollapsed ? i18n.t('common.about') : ''"
        >
          <Info class="nav-icon" :size="20" :stroke-width="1.8" />
          <transition name="fade">
            <span v-if="!ui.sidebarCollapsed" class="nav-label">{{ i18n.t("common.about") }}</span>
          </transition>
        </div>
      </div>
    </nav>

    <div class="sidebar-footer">
      <div class="nav-item collapse-btn" @click="ui.toggleSidebar()">
        <ChevronLeft
          class="nav-icon"
          :style="{ transform: ui.sidebarCollapsed ? 'rotate(180deg)' : '' }"
          :size="20"
          :stroke-width="1.8"
        />
        <transition name="fade">
          <span v-if="!ui.sidebarCollapsed" class="nav-label">{{
            i18n.t("sidebar.collapse_btn")
          }}</span>
        </transition>
      </div>
    </div>
  </aside>
</template>

<style src="@styles/components/layout/AppSidebar.css" scoped></style>
