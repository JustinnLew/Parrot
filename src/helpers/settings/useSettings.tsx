import {
  createContext, useCallback, useContext, useEffect, useMemo, useState,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";

export type AppSettings = { source_lang: string; target_lang: string };

type SettingsValue = {
  settings: AppSettings | null; // null until the first fetch finishes
  setSourceLang: (lang: string) => Promise<void>;
  setTargetLang: (lang: string) => Promise<void>;
};

const SettingsContext = createContext<SettingsValue | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings | null>(null);

  useEffect(() => {
    invoke<AppSettings>("settings_get").then(setSettings).catch(console.error);
  }, []);

  const setSourceLang = useCallback(async (sourceLang: string) => {
    setSettings(await invoke<AppSettings>("settings_set_source_lang", { sourceLang }));
  }, []);

  const setTargetLang = useCallback(async (targetLang: string) => {
    setSettings(await invoke<AppSettings>("settings_set_target_lang", { targetLang }));
  }, []);

  const value = useMemo(
    () => ({ settings, setSourceLang, setTargetLang }),
    [settings, setSourceLang, setTargetLang]
  );

  return <SettingsContext.Provider value={value}>{children}</SettingsContext.Provider>;
}

export function useSettings() {
  const ctx = useContext(SettingsContext);
  if (!ctx) throw new Error("useSettings must be used inside <SettingsProvider>");
  return ctx;
}