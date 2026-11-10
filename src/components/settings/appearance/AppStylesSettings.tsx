import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { AppStyle, WritingStyle } from "@/bindings";
import { useSettings } from "@/hooks/useSettings";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { SettingContainer } from "../../ui/SettingContainer";
import { BareGroup } from "../../ui/BareGroup";
import { SegmentedControl } from "./SegmentedControl";

const STYLES: WritingStyle[] = ["none", "formal", "casual", "code"];

/** Per-app writing styles: bundle id plus a style for each row. */
/** `bare` drops the titled card so a parent (Smart features) can frame it. */
export const AppStylesSettings: React.FC<{ bare?: boolean }> = ({ bare = false }) => {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettings();
  const [rows, setRows] = useState<AppStyle[]>([]);

  useEffect(() => {
    setRows(settings?.app_styles ?? []);
  }, [settings?.app_styles]);

  if (!settings) return null;
  const b = "settings.appStyles";
  const options = STYLES.map((value) => ({
    value,
    label: t(`${b}.style.${value}`),
  }));

  const save = (next: AppStyle[]) => updateSetting("app_styles", next);
  const edit = (i: number, patch: Partial<AppStyle>) =>
    setRows((prev) => prev.map((r, k) => (k === i ? { ...r, ...patch } : r)));

  const field =
    "min-h-[30px] rounded-[10px] bg-wash px-2 text-xs outline-none focus:outline-none";

  const Group = bare ? BareGroup : SettingsGroup;

  return (
    <Group title={t(`${b}.title`)}>
      <SettingContainer
        title={t(`${b}.list.title`)}
        description={t(`${b}.list.description`)}
        descriptionMode="inline"
        grouped={true}
        layout="stacked"
      >
        <div className="flex w-full flex-col gap-2">
          {rows.map((row, i) => (
            <div key={i} className="flex flex-wrap items-center gap-2">
              <input
                className={`${field} min-w-0 flex-1`}
                placeholder={t(`${b}.bundleId`)}
                aria-label={t(`${b}.bundleId`)}
                value={row.bundle_id}
                spellCheck={false}
                onChange={(e) => edit(i, { bundle_id: e.target.value })}
                onBlur={() => save(rows)}
              />
              <SegmentedControl
                ariaLabel={t(`${b}.list.title`)}
                options={options}
                value={row.style}
                onChange={(v) => {
                  const next = rows.map((r, k) =>
                    k === i ? { ...r, style: v as WritingStyle } : r,
                  );
                  setRows(next);
                  // A row without a bundle id is saved once one is typed.
                  return row.bundle_id.trim() ? save(next) : undefined;
                }}
              />
              <button
                type="button"
                aria-label={t(`${b}.remove`)}
                title={t(`${b}.remove`)}
                onClick={() => {
                  const next = rows.filter((_, k) => k !== i);
                  setRows(next);
                  save(next);
                }}
                className="h-[30px] w-[30px] rounded-[10px] text-center text-ink-2 hover:bg-wash active:scale-[0.97] outline-none focus:outline-none"
              >
                ×
              </button>
            </div>
          ))}
          <button
            type="button"
            onClick={() =>
              setRows((prev) => [...prev, { bundle_id: "", style: "formal" }])
            }
            className="self-start min-h-[30px] px-3 rounded-[10px] bg-wash text-xs font-medium text-center hover:bg-line active:scale-[0.97] outline-none focus:outline-none"
          >
            {t(`${b}.add`)}
          </button>
        </div>
      </SettingContainer>
    </Group>
  );
};
