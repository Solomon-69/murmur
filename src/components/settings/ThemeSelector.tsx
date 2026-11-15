import React from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../ui/SettingContainer";
import { SegmentedControl } from "./appearance/SegmentedControl";
import { useSettings } from "@/hooks/useSettings";
import {
  applyTheme,
  resolveTheme,
  THEME_OPTIONS,
  type AppTheme,
} from "@/lib/utils/theme";

interface ThemeSelectorProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

/** System / Light / Dark. Choosing any of them also turns the old Black theme off. */
export const ThemeSelector: React.FC<ThemeSelectorProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { settings, updateSetting } = useSettings();

    const current = resolveTheme(settings?.theme, settings?.black_theme);

    const options = THEME_OPTIONS.map((value) => ({
      value,
      label: t(`theme.options.${value}`),
    }));

    const handleChange = async (value: string) => {
      const next = (value === "black" ? "dark" : value) as AppTheme;
      applyTheme(next);
      await updateSetting("black_theme", false);
      await updateSetting("theme", next);
    };

    return (
      <SettingContainer
        title={t("theme.title")}
        description={t("theme.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      >
        <SegmentedControl
          ariaLabel={t("theme.title")}
          options={options}
          value={current}
          onChange={handleChange}
        />
      </SettingContainer>
    );
  },
);

ThemeSelector.displayName = "ThemeSelector";
