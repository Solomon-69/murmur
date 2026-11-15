import React from "react";
import { useTranslation } from "react-i18next";
import { AppWindow, Keyboard, Lock, Languages } from "lucide-react";
import { Button } from "../../ui/Button";
import { Mascot } from "../../shared/Mascot";
import MurmurTextLogo from "../../icons/MurmurTextLogo";
import { OnboardingShell, ScreenHeader } from "../OnboardingShell";

interface IntroProps {
  progress: { index: number; count: number };
  onContinue: () => void;
}

const FEATURES = [
  { key: "shortcut", Icon: Keyboard },
  { key: "anywhere", Icon: AppWindow },
  { key: "private", Icon: Lock },
  { key: "languages", Icon: Languages },
] as const;

/** Screen 0: what murmur is and what it does, before any setup. */
export const Intro: React.FC<IntroProps> = ({ progress, onContinue }) => {
  const { t } = useTranslation();
  return (
    <OnboardingShell
      progress={progress}
      primary={
        <Button variant="primary" size="lg" onClick={onContinue} autoFocus>
          {t("onboarding.intro.start")}
        </Button>
      }
    >
      <div className="intro flex h-full min-h-0 flex-col items-center justify-center-safe px-gutter pb-[8px] text-center">
        <div className="intro-in mb-[14px] inline-flex items-end gap-[10px] leading-none">
          <Mascot size={72} animation="wave" />
          <MurmurTextLogo width={150} className="mb-[6px] block" />
        </div>
        <div className="intro-in" style={{ animationDelay: "90ms" }}>
          <ScreenHeader
            title={t("onboarding.intro.title")}
            sub={t("onboarding.intro.subtitle")}
          />
        </div>
        <ul className="m-0 mt-[26px] grid w-full max-w-[560px] list-none grid-cols-2 gap-[10px] p-0 text-start">
          {FEATURES.map(({ key, Icon }, i) => (
            <li
              key={key}
              className="intro-in intro-card flex items-start gap-[12px] rounded-panel border-[1.5px] border-line bg-background px-[14px] py-[12px]"
              style={{ animationDelay: `${200 + i * 70}ms` }}
            >
              <span className="intro-icon mt-[2px] grid size-[32px] flex-none place-items-center rounded-control bg-wash text-text">
                <Icon size={17} strokeWidth={2} aria-hidden="true" />
              </span>
              <span className="min-w-0">
                <span className="block text-body leading-[1.35] font-semibold text-text">
                  {t(`onboarding.intro.features.${key}.title`)}
                </span>
                <span className="mt-[2px] block text-small leading-[1.4] text-ink-2">
                  {t(`onboarding.intro.features.${key}.body`)}
                </span>
              </span>
            </li>
          ))}
        </ul>
      </div>
    </OnboardingShell>
  );
};
