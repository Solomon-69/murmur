/** The tray icon, traced from src-tauri/resources/tray_*.png (the murmur wave); uses currentColor like a macOS template image. */
type TrayIconState = "idle" | "recording" | "transcribing";

/** [cx, rx, ry] for each bar of the wave, measured from the 64px tray PNGs. */
const WAVE: ReadonlyArray<readonly [number, number, number]> = [
  [4, 2, 2],
  [12, 3, 4],
  [20.5, 4.5, 9],
  [32, 6, 17],
  [43.5, 4.5, 9],
  [52, 3, 4],
  [60, 2, 2],
];

const Wave = ({ cy, opacity = 1 }: { cy: number; opacity?: number }) => (
  <g opacity={opacity}>
    {WAVE.map(([cx, rx, ry]) => (
      <ellipse key={cx} cx={cx} cy={cy} rx={rx} ry={ry} />
    ))}
  </g>
);

const TrayIcon = ({
  state,
  size,
}: {
  state: TrayIconState;
  size: number | string;
}) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 64 64"
    className="fill-current"
    xmlns="http://www.w3.org/2000/svg"
  >
    {state === "idle" && <Wave cy={32} />}
    {state === "recording" && (
      <>
        <Wave cy={32} />
        <circle cx={55} cy={8} r={6} />
      </>
    )}
    {state === "transcribing" && (
      <>
        <Wave cy={29.5} opacity={0.45} />
        <circle cx={21} cy={58} r={2.5} />
        <circle cx={32} cy={58} r={2.5} />
        <circle cx={43} cy={58} r={2.5} />
      </>
    )}
  </svg>
);

export default TrayIcon;
