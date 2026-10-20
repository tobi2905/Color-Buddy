interface SwatchProps {
  hex: string;
  /** Pixel size of a square swatch; ignored when className sets the size. */
  size?: number;
  className?: string;
  /** Spoken label; omit for decorative swatches next to visible text. */
  label?: string;
}

/** A colour sample. Always paired with visible text elsewhere (never colour alone). */
export function Swatch({ hex, size = 20, className = "", label }: SwatchProps) {
  return (
    <span
      className={`inline-block shrink-0 rounded border border-line ${className}`}
      style={{ backgroundColor: hex, ...(className ? {} : { width: size, height: size }) }}
      {...(label ? { role: "img", "aria-label": label } : { "aria-hidden": true })}
    />
  );
}
