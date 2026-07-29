interface DatePickerProps {
  value: string;
  onChange: (date: string) => void;
  className?: string;
}

export default function DatePicker({ value, onChange, className = "" }: DatePickerProps) {
  return (
    <input
      type="date"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className={`border rounded px-2 py-1 ${className}`}
    />
  );
}
