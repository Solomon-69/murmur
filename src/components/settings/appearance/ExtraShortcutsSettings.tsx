import React from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { useSettings } from "@/hooks/useSettings";
import { notify } from "@/stores/noticeStore";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { ToggleSwitch } from "../../ui/ToggleSwitch";
import { ShortcutInput } from "../ShortcutInput";

/** Optional shortcut: paste last dictation (off by default). Command Mode lives in Smart features. */
export const ExtraShortcutsSettings: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettings();
  if (!settings) return null;
  const b = "settings.extraShortcuts";
  const pasteLast = settings.paste_last_shortcut_enabled ?? false;

  const pasteNow = async () => {
    const result = await commands.pasteLastDictation();
    if (result.status === "error") notify({ title: result.error });
  };

  return (
    <SettingsGroup title={t(`${b}.title`)}>
      <ToggleSwitch
        checked={pasteLast}
        onChange={(v) => updateSetting("paste_last_shortcut_enabled", v)}
        label={t(`${b}.pasteLast.title`)}
        description={t(`${b}.pasteLast.description`)}
        grouped={true}
      />
      {pasteLast && <ShortcutInput shortcutId="paste_last" grouped={true} />}
      <div className="px-4 py-2">
        <button
          type="button"
          onClick={pasteNow}
          className="min-h-[30px] px-3 rounded-[10px] bg-wash text-xs font-medium text-center hover:bg-line transition-transform duration-150 active:scale-[0.97] outline-none focus:outline-none"
        >
          {t(`${b}.pasteNow`)}
        </button>
      </div>
    </SettingsGroup>
  );
};
