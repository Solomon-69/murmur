import React, { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands, events, type UsageStats } from "@/bindings";
import { useCountUp } from "@/hooks/useCountUp";
import { SettingsGroup } from "../../ui/SettingsGroup";

const StatTile: React.FC<{ label: string; value: number | null; empty: string }> = ({
  label,
  value,
  empty,
}) => {
  const { i18n } = useTranslation();
  const shown = useCountUp(value ?? 0);
  return (
    <div className="flex flex-col gap-1 rounded-[10px] bg-wash px-3 py-3">
      <span className="text-xl font-semibold tabular-nums text-text">
        {value === null ? empty : shown.toLocaleString(i18n.language)}
      </span>
      <span className="text-xs text-ink-2">{label}</span>
    </div>
  );
};

/** Words, speed and streak from local history, counting up into place. */
export const UsageStatsCard: React.FC = () => {
  const { t } = useTranslation();
  const [stats, setStats] = useState<UsageStats | null>(null);
  const b = "settings.stats";

  const load = useCallback(async () => {
    const result = await commands.getUsageStats();
    if (result.status === "ok") setStats(result.data);
  }, []);

  useEffect(() => {
    load();
    const unlisten = events.historyUpdatePayload.listen(() => {
      load();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [load]);

  if (!stats) return null;
  const empty = t(`${b}.noValue`);

  return (
    <SettingsGroup title={t(`${b}.title`)}>
      <div className="px-4 py-3">
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
          <StatTile label={t(`${b}.totalWords`)} value={stats.total_words} empty={empty} />
          <StatTile label={t(`${b}.thisWeek`)} value={stats.words_this_week} empty={empty} />
          <StatTile label={t(`${b}.wpm`)} value={stats.words_per_minute} empty={empty} />
          <StatTile label={t(`${b}.streak`)} value={stats.day_streak} empty={empty} />
        </div>
        <p className="pt-2 text-xs text-ink-3">{t(`${b}.note`)}</p>
      </div>
    </SettingsGroup>
  );
};
