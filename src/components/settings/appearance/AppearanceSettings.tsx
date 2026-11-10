import React from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { useSettings } from "@/hooks/useSettings";
import { normalizeAccent } from "@/lib/constants/accents";
import { useModelStore } from "@/stores/modelStore";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { SettingContainer } from "../../ui/SettingContainer";
import { ThemeSelector } from "../ThemeSelector";
import { SegmentedControl } from "./SegmentedControl";
import { ToggleSwitch } from "../../ui/ToggleSwitch";
import {
  BarPreview,
  CLEAR_TINTS,
  GLASS_TINTS,
} from "./BarPreview";
import { TintSwatches } from "./TintSwatches";
import { BLACK_SWATCH_COLORS, CLEAR_SWATCH_COLORS } from "./ColorSwatch";
import { AccentPicker } from "./AccentPicker";

const TINTS = Object.keys(GLASS_TINTS);
const CLEAR_IDS = Object.keys(CLEAR_TINTS);

export const AppearanceSettings: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettings();
  const showsWords = useModelStore(
    (m) =>
      m.models.find((x) => x.id === m.currentModel)?.supports_streaming ??
      false,
  );
  if (!settings) return null;

  const style = settings.overlay_style ?? "live";
  const size = settings.overlay_size ?? "compact";
  const position = settings.overlay_position ?? "bottom";
  const horizontal = settings.overlay_horizontal ?? "screen";
  const offset = settings.overlay_offset_x ?? 0;
  const accent = normalizeAccent(settings.overlay_accent);
  const opacity = settings.overlay_opacity ?? 1;
  const glass = settings.overlay_glass ?? "clear";
  const tint = settings.overlay_glass_tint ?? "black";
  const clearTint = settings.overlay_clear_tint ?? "clear";

  const seg = (base: string, values: string[]) =>
    values.map((value) => ({ value, label: t(`${base}.${value}`) }));
  const a = "settings.appearance";

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t(`${a}.sections.app`)}>
        <ThemeSelector descriptionMode="tooltip" grouped={true} />
      </SettingsGroup>
      <SettingsGroup title={t(`${a}.sections.bar`)}>
        <div className="px-4 py-4 flex flex-col items-center gap-3">
          <BarPreview
            sample={t(`${a}.sample`)}
            style={style}
            size={size}
            accent={accent}
            glass={glass}
            tint={tint}
            clearTint={clearTint}
            opacity={opacity}
            position={position}
            showsWords={showsWords}
          />
          <button
            type="button"
            onClick={() => commands.previewRecordingOverlay()}
            className="inline-flex min-h-[32px] items-center justify-center px-4 rounded-full text-xs font-medium text-center bg-wash hover:bg-line transition-[background-color,transform] duration-150 active:scale-[0.97] outline-none focus:outline-none"
          >
            {t(`${a}.showOnScreen`)}
          </button>
        </div>
        <ToggleSwitch
          checked={style !== "none"}
          onChange={(on) =>
            updateSetting(
              "overlay_style",
              (on ? "live" : "none") as typeof style,
            )
          }
          label={t(`${a}.barStyle.title`)}
          description={t(`${a}.barStyle.description`)}
          descriptionMode="tooltip"
          grouped={true}
        />
        <SettingContainer
          title={t(`${a}.glass.title`)}
          description={t(`${a}.glass.description`)}
          descriptionMode="tooltip"
          grouped={true}
        >
          <SegmentedControl
            ariaLabel={t(`${a}.glass.title`)}
            options={seg(`${a}.glass`, ["clear", "black"])}
            value={glass}
            onChange={(v) => updateSetting("overlay_glass", v)}
          />
        </SettingContainer>
        {glass === "clear" && (
          <SettingContainer
            title={t(`${a}.clearTint.title`)}
            description={t(`${a}.clearTint.description`)}
            descriptionMode="tooltip"
            grouped={true}
          >
            <TintSwatches
              ids={CLEAR_IDS}
              ring="dark"
              value={clearTint}
              label={t(`${a}.clearTint.title`)}
              name={(id) => t(`${a}.clearTint.${id}`)}
              fill={(id) => CLEAR_SWATCH_COLORS[id] ?? CLEAR_SWATCH_COLORS.clear}
              onChange={(id) => updateSetting("overlay_clear_tint", id)}
            />
          </SettingContainer>
        )}
        {glass === "black" && (
          <SettingContainer
            title={t(`${a}.tint.title`)}
            description={t(`${a}.tint.description`)}
            descriptionMode="tooltip"
            grouped={true}
          >
            <TintSwatches
              ids={TINTS}
              value={tint}
              label={t(`${a}.tint.title`)}
              name={(id) => t(`${a}.tint.${id}`)}
              fill={(id) => BLACK_SWATCH_COLORS[id] ?? BLACK_SWATCH_COLORS.black}
              onChange={(id) => updateSetting("overlay_glass_tint", id)}
            />
          </SettingContainer>
        )}
      </SettingsGroup>
      <SettingsGroup title={t(`${a}.sections.barPlace`)}>
        <SettingContainer
          title={t(`${a}.size.title`)}
          description={t(`${a}.size.description`)}
          descriptionMode="tooltip"
          grouped={true}
        >
          <SegmentedControl
            ariaLabel={t(`${a}.size.title`)}
            options={seg(`${a}.size`, ["compact", "regular"])}
            value={size}
            onChange={(v) => updateSetting("overlay_size", v as typeof size)}
          />
        </SettingContainer>
        <SettingContainer
          title={t(`${a}.position.title`)}
          description={t(`${a}.position.description`)}
          descriptionMode="tooltip"
          grouped={true}
        >
          <SegmentedControl
            ariaLabel={t(`${a}.position.title`)}
            options={seg(`${a}.position`, ["top", "bottom"])}
            value={position}
            onChange={(v) =>
              updateSetting("overlay_position", v as typeof position)
            }
          />
        </SettingContainer>
        <SettingContainer
          title={t(`${a}.horizontal.title`)}
          description={t(`${a}.horizontal.description`)}
          descriptionMode="tooltip"
          grouped={true}
        >
          <SegmentedControl
            ariaLabel={t(`${a}.horizontal.title`)}
            options={seg(`${a}.horizontal`, [
              "screen",
              "active_window",
              "custom",
            ])}
            value={horizontal}
            onChange={(v) =>
              updateSetting("overlay_horizontal", v as typeof horizontal)
            }
          />
        </SettingContainer>
        {horizontal === "custom" && (
          <SettingContainer
            title={t(`${a}.offset.title`)}
            description={t(`${a}.offset.description`)}
            descriptionMode="tooltip"
            grouped={true}
          >
            <div className="flex items-center gap-2">
              <input
                type="range"
                min={-600}
                max={600}
                step={1}
                defaultValue={offset}
                aria-label={t(`${a}.offset.title`)}
                onChange={(e) =>
                  updateSetting("overlay_offset_x", Number(e.target.value))
                }
                className="w-40 accent-logo-primary"
              />
              <span className="w-12 text-right text-xs tabular-nums text-ink-2">
                {Math.round(offset)}
              </span>
            </div>
          </SettingContainer>
        )}
        <SettingContainer
          title={t(`${a}.accent.title`)}
          description={t(`${a}.accent.description`)}
          descriptionMode="tooltip"
          layout="stacked"
          grouped={true}
        >
          <AccentPicker
            value={accent}
            onChange={(id) => updateSetting("overlay_accent", id)}
          />
        </SettingContainer>
        <SettingContainer
          title={t(`${a}.opacity.title`)}
          description={t(`${a}.opacity.description`)}
          descriptionMode="tooltip"
          grouped={true}
        >
          <div className="flex items-center gap-2">
            <input
              type="range"
              min={40}
              max={100}
              step={5}
              defaultValue={Math.round(opacity * 100)}
              aria-label={t(`${a}.opacity.title`)}
              onChange={(e) =>
                updateSetting("overlay_opacity", Number(e.target.value) / 100)
              }
              className="w-40 accent-logo-primary"
            />
            <span className="w-12 text-right text-xs tabular-nums text-ink-2">
              {Math.round(opacity * 100)}%
            </span>
          </div>
        </SettingContainer>
      </SettingsGroup>
    </div>
  );
};
