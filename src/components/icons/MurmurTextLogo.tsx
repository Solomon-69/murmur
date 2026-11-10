import { useId } from "react";

/** Grey bands of the murmur ribbon, darkest first. Light theme uses ink, dark theme uses white. */
const GREYS_LIGHT = ["#0b0b0c", "#38383c", "#68686d", "#9b9ba0", "#cbcbce"];
const GREYS_DARK = ["#ffffff", "#d2d2d5", "#a2a2a6", "#727277", "#47474c"];

/** Each band reads a theme.css variable (--mw-l0..4 / --mw-d0..4) that is only
 *  set when an accent other than Grey is picked, so the greys are the fallback
 *  and the logo follows the accent live, with no re-render. */
const bands = (prefix: string, greys: string[]) =>
  greys.map((grey, i) => `var(--mw-${prefix}${i}, ${grey})`);
const BANDS_LIGHT = bands("l", GREYS_LIGHT);
const BANDS_DARK = bands("d", GREYS_DARK);

const W = 420;
const H = 130;
const CY = 74;

/** Brand name, never translated. */
const NAME = "murmur";

const round = (n: number) => Math.round(n * 100) / 100;

/** Wavy band edges that fill the letters, like the layers of the ribbon logo. */
const bandPath = (i: number) => {
  let d = "";
  for (let x = 0; x <= W; x += 6) {
    const y = CY - 30 + i * 13 + 5 * Math.sin((x / W) * Math.PI * 3 + i * 0.5);
    d += `${x === 0 ? "M" : "L"}${x} ${round(y)}`;
  }
  return `${d}L${W} ${H} L0 ${H}Z`;
};

const BAND_PATHS = [1, 2, 3, 4].map(bandPath);

/** Draws one colour version; the theme decides which one shows. */
const Wordmark = ({ bands, id }: { bands: string[]; id: string }) => (
  <>
    <defs>
      <clipPath id={id}>
        <text
          x="16"
          y={CY + 26}
          textLength="388"
          lengthAdjust="spacingAndGlyphs"
          fontFamily="'Murmur Wordmark', 'Baloo 2', sans-serif"
          fontWeight={800}
          fontSize="86"
          letterSpacing="-2"
        >
          {NAME}
        </text>
      </clipPath>
    </defs>
    <g clipPath={`url(#${id})`}>
      <rect width={W} height={H} style={{ fill: bands[0] }} />
      {BAND_PATHS.map((d, i) => (
        <path key={i} d={d} style={{ fill: bands[i + 1] }} />
      ))}
    </g>
  </>
);

/** murmur wordmark: the name filled with the ribbon's grey bands. Kept under the old
 *  component name so every place that showed the logo picks it up. */
const MurmurTextLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  const id = useId().replace(/:/g, "");
  return (
    <svg
      width={width}
      height={height}
      className={className}
      viewBox={`0 0 ${W} ${H}`}
      role="img"
      aria-label="murmur"
      xmlns="http://www.w3.org/2000/svg"
    >
      <g className="mw-light">
        <Wordmark bands={BANDS_LIGHT} id={`${id}l`} />
      </g>
      <g className="mw-dark">
        <Wordmark bands={BANDS_DARK} id={`${id}d`} />
      </g>
    </svg>
  );
};

export default MurmurTextLogo;
