import React from "react";
import { useTranslation } from "react-i18next";
import { AppWindow, Keyboard, Lock, Languages } from "lucide-react";
import { Button } from "../../ui/Button";
import { Mascot } from "../../shared/Mascot";
import MurmurTextLogo from "../../icons/MurmurTextLogo";
import { OnboardingShell } from "../OnboardingShell";

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
      <div className="flex h-full min-h-0 flex-col items-center px-[28px] pt-[8px] pb-[10px] text-center">
        <div className="intro-in flex items-center gap-[14px] leading-none">
          <Mascot size={68} animation="wave" />
          <MurmurTextLogo width={168} className="block" />
        </div>
        <h1
          className="intro-in m-0 mt-[18px] font-display text-[32px] leading-[1.1] font-[680] tracking-[-0.022em] text-balance text-text"
          style={{ animationDelay: "80ms" }}
        >
          {t("onboarding.intro.title")}
        </h1>
        <p
          className="intro-in mt-[10px] mb-0 max-w-[520px] text-[16px] leading-[1.5] text-pretty text-ink-2"
          style={{ animationDelay: "140ms" }}
        >
          {t("onboarding.intro.subtitle")}
        </p>
        <ul className="m-0 mt-[22px] grid min-h-0 w-full flex-1 list-none grid-cols-2 grid-rows-2 gap-[12px] p-0 text-start">
          {FEATURES.map(({ key, Icon }, i) => (
            <li
              key={key}
              className="intro-in intro-card flex min-h-0 items-center gap-[14px] rounded-panel border-[1.5px] border-line bg-wash/40 px-[18px] py-[12px]"
              style={{ animationDelay: `${220 + i * 70}ms` }}
            >
              <span className="intro-icon grid size-[40px] flex-none place-items-center rounded-control bg-background-ui text-background">
                <Icon size={20} strokeWidth={2} aria-hidden="true" />
              </span>
              <span className="min-w-0">
                <span className="block text-[16px] leading-[1.3] font-semibold text-text">
                  {t(`onboarding.intro.features.${key}.title`)}
                </span>
                <span className="mt-[3px] block text-small leading-[1.4] text-ink-2">
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
