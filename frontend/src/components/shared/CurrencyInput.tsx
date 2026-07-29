import { useState, useCallback } from "react";

function centsToDisplay(cents: number): string {
  const abs = Math.abs(cents);
  const s = `${(abs / 100).toFixed(2)}`;
  return cents < 0 ? `-${s}` : s;
}

function displayToCents(display: string): number {
  const cleaned = display.replace(/[^0-9.\-]/g, "");
  const val = parseFloat(cleaned);
  if (isNaN(val)) return 0;
  return Math.round(val * 100);
}

interface CurrencyInputProps {
  value: number;
  onChange: (cents: number) => void;
  className?: string;
}

export default function CurrencyInput({ value, onChange, className = "" }: CurrencyInputProps) {
  const [focused, setFocused] = useState(false);
  const [draft, setDraft] = useState("");

  const displayValue = focused ? draft : centsToDisplay(value);

  const handleFocus = useCallback(() => {
    setFocused(true);
    setDraft(centsToDisplay(value));
  }, [value]);

  const handleBlur = useCallback(() => {
    setFocused(false);
    onChange(displayToCents(draft));
  }, [draft, onChange]);

  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value;
    if (/^[0-9.\-]*$/.test(raw)) {
      setDraft(raw);
    }
  }, []);

  return (
    <input
      type="text"
      inputMode="decimal"
      value={displayValue}
      onFocus={handleFocus}
      onBlur={handleBlur}
      onChange={handleChange}
      className={`border rounded px-2 py-1 text-right ${className}`}
    />
  );
}
