import { ref, type Ref } from "vue";

const languageFiles: Record<string, any> = import.meta.glob("./*.json", { eager: true });

const processLanguageFiles = () => {
  const translations: Record<string, LanguageFile> = {};
  const supportedLocales: string[] = [];

  for (const [path, module] of Object.entries(languageFiles)) {
    const match = path.match(/\.\/(.*)\.json$/);
    if (match) {
      const localeCode = match[1];
      const data = (module as any).default;

      if (data && typeof data === "object") {
        translations[localeCode] = data;
        supportedLocales.push(localeCode);
      }
    }
  }

  return { translations, supportedLocales };
};

type TranslationNode = {
  [key: string]: string | TranslationNode;
};

type LanguageFile = TranslationNode & {
  languageName?: string;
};

const { translations, supportedLocales } = processLanguageFiles();

export const SUPPORTED_LOCALES: readonly string[] = supportedLocales;
export type LocaleCode = string;

export function setTranslations(locale: LocaleCode, data: LanguageFile) {
  if (isSupportedLocale(locale)) {
    translations[locale] = data;
  }
}

function isSupportedLocale(locale: string): locale is LocaleCode {
  return supportedLocales.includes(locale);
}

function resolveNestedValue(source: TranslationNode, keys: string[]): string | undefined {
  let current: string | TranslationNode | undefined = source;
  for (const key of keys) {
    if (!current || typeof current === "string") {
      return undefined;
    }
    current = current[key];
  }

  return typeof current === "string" ? current : undefined;
}

function interpolateVariables(template: string, options: Record<string, unknown>): string {
  return template
    .replace(/\{\{([^}]+)\}\}/g, (match, varName) => {
      const value = options[varName.trim()];
      return value === undefined || value === null ? match : String(value);
    })
    .replace(/\{([^}]+)\}/g, (match, varName) => {
      const value = options[varName.trim()];
      return value === undefined || value === null ? match : String(value);
    });
}

class I18n {
  private currentLocale: Ref<LocaleCode> = ref("zh-CN");
  private fallbackLocale: LocaleCode = "en-US";

  setLocale(locale: string) {
    if (isSupportedLocale(locale)) {
      this.currentLocale.value = locale;
    }
  }

  getLocale(): LocaleCode {
    return this.currentLocale.value;
  }

  t(key: string, options: Record<string, unknown> = {}): string {
    const keys = key.split(".");
    const currentLocaleValue = this.currentLocale.value;

    let resolved: string | undefined =
      resolveNestedValue(translations[currentLocaleValue], ["sealantern"].concat(keys)) ??
      resolveNestedValue(translations[currentLocaleValue], keys) ??
      resolveNestedValue(translations[this.fallbackLocale], ["sealantern"].concat(keys)) ??
      resolveNestedValue(translations[this.fallbackLocale], keys);

    if (resolved === undefined) {
      return key;
    }

    return interpolateVariables(resolved, options);
  }

  te(key: string): boolean {
    const keys = key.split(".");
    const currentLocaleValue = this.currentLocale.value;
    const resolved =
      resolveNestedValue(translations[currentLocaleValue], ["sealantern"].concat(keys)) ??
      resolveNestedValue(translations[currentLocaleValue], keys) ??
      resolveNestedValue(translations[this.fallbackLocale], ["sealantern"].concat(keys)) ??
      resolveNestedValue(translations[this.fallbackLocale], keys);
    return resolved !== undefined;
  }

  getTranslations() {
    return translations as Record<string, LanguageFile>;
  }

  getLocaleRef() {
    return this.currentLocale;
  }

  getAvailableLocales(): readonly LocaleCode[] {
    return supportedLocales;
  }

  isSupportedLocale(locale: string): boolean {
    return (SUPPORTED_LOCALES as readonly string[]).includes(locale);
  }
}

export const i18n = new I18n();

const languageAPI = {
  i18n,
  SUPPORTED_LOCALES,
  setTranslations,
};

export default languageAPI;
