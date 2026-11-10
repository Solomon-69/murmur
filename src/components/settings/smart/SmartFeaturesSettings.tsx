import React from "react";
import { useTranslation } from "react-i18next";
import {
  BookA,
  Hand,
  MessageSquareQuote,
  Palette,
  Sparkles,
  Wand2,
} from "lucide-react";
import { useSettings } from "@/hooks/useSettings";
import { SettingContainer } from "../../ui/SettingContainer";
import { ToggleSwitch } from "../../ui/ToggleSwitch";
import { ShortcutActivationSetting } from "../ShortcutActivation";
import { CustomWords } from "../CustomWords";
import { ShortcutInput } from "../ShortcutInput";
import { SnippetsSettings } from "../appearance/SnippetsSettings";
import { AppStylesSettings } from "../appearance/AppStylesSettings";
import { SegmentedControl } from "../appearance/SegmentedControl";

interface FeatureCardProps {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  what: string;
  example: string;
  exampleLabel: string;
  children: React.ReactNode;
}

/** One smart feature: what it does, a small example, then its controls. */
const FeatureCard: React.FC<FeatureCardProps> = ({
  icon: Icon,
  title,
  what,
  example,
  exampleLabel,
  children,
}) => (
  <section className="overflow-visible rounded-[16px] border border-line bg-card shadow-[0_1px_2px_rgba(0,0,0,0.04)]">
    <header className="flex gap-3 px-4 pt-4 pb-3">
      <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[10px] bg-logo-primary/12 text-logo-primary">
        <Icon className="h-[18px] w-[18px]" />
      </span>
      <div className="min-w-0 flex-1">
        <h2 className="text-[15px] font-semibold leading-tight">{title}</h2>
        <p className="mt-1 text-[13px] leading-snug text-ink-2">{what}</p>
        <p className="mt-2 inline-flex max-w-full items-baseline gap-1.5 rounded-full bg-wash px-3 py-1 text-xs text-ink-2">
          <span className="font-semibold text-text">{exampleLabel}</span>
          <span className="font-transcript">{example}</span>
        </p>
      </div>
    </header>
    <div className="settings-rows border-t border-line">{children}</div>
  </section>
);

/**
 * The Wispr Flow-style features, gathered in one place so they are easy to
 * find: hands-free, personal dictionary, snippets, per-app styles, command
 * mode and smart prompt mode. Same settings keys as before; only moved.
 */
export const SmartFeaturesSettings: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettings();
  if (!settings) return null;

  const s = "settings.smart";
  const chip = "settings.appearance.promptChip";
  const exampleLabel = t(`${s}.example`);
  const chipMode = settings.prompt_chip_mode ?? "smart";
  const minWords = settings.prompt_chip_min_words ?? 12;
  const commandMode = settings.command_mode_enabled ?? false;

  const card = (key: string) => ({
    title: t(`${s}.${key}.title`),
    what: t(`${s}.${key}.what`),
    example: t(`${s}.${key}.example`),
    exampleLabel,
  });

  return (
    <div className="max-w-3xl w-full mx-auto space-y-5">
      <div className="px-1">
        <h1 className="text-xl font-semibold">{t(`${s}.title`)}</h1>
        <p className="mt-1 text-[13px] text-ink-2">{t(`${s}.intro`)}</p>
      </div>

      <FeatureCard icon={Sparkles} {...card("prompt")}>
        <SettingContainer
          title={t(`${chip}.title`)}
          description={t(`${chip}.description`)}
          descriptionMode="tooltip"
          grouped={true}
        >
          <SegmentedControl
            ariaLabel={t(`${chip}.title`)}
            options={["off", "smart", "ask", "always"].map((value) => ({
              value,
              label: t(`${chip}.${value}`),
            }))}
            value={chipMode}
            onChange={(v) =>
              updateSetting("prompt_chip_mode", v as typeof chipMode)
            }
          />
        </SettingContainer>
        {(chipMode === "ask" || chipMode === "always") && (
          <SettingContainer
            title={t(`${chip}.minWords`)}
            description={t(`${chip}.description`)}
            descriptionMode="tooltip"
            grouped={true}
          >
            <input
              type="number"
              min={5}
              max={2000}
              defaultValue={minWords}
              aria-label={t(`${chip}.minWords`)}
              onBlur={(e) =>
                updateSetting(
                  "prompt_chip_min_words",
                  Number(e.target.value) || 12,
                )
              }
              className="w-20 min-h-[30px] rounded-[10px] bg-wash px-2 text-center text-xs outline-none focus:outline-none"
            />
          </SettingContainer>
        )}
      </FeatureCard>

      <FeatureCard icon={Hand} {...card("handsFree")}>
        <ShortcutActivationSetting descriptionMode="tooltip" grouped={true} />
      </FeatureCard>

      <FeatureCard icon={BookA} {...card("dictionary")}>
        <CustomWords descriptionMode="tooltip" grouped />
      </FeatureCard>

      <FeatureCard icon={MessageSquareQuote} {...card("snippets")}>
        <SnippetsSettings bare />
      </FeatureCard>

      <FeatureCard icon={Palette} {...card("appStyles")}>
        <AppStylesSettings bare />
      </FeatureCard>

      <FeatureCard icon={Wand2} {...card("commandMode")}>
        <ToggleSwitch
          checked={commandMode}
          onChange={(v) => updateSetting("command_mode_enabled", v)}
          label={t("settings.extraShortcuts.commandMode.title")}
          description={t("settings.extraShortcuts.commandMode.description")}
          grouped={true}
        />
        {commandMode && (
          <ShortcutInput shortcutId="command_mode" grouped={true} />
        )}
      </FeatureCard>

    </div>
  );
};
