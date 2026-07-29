import { useState, useCallback, useRef, useEffect } from "react";

interface Account {
  id: number;
  name: string;
}

interface AccountSelectProps {
  accounts: Account[];
  value: string;
  onChange: (name: string) => void;
  className?: string;
}

export default function AccountSelect({ accounts, value, onChange, className = "" }: AccountSelectProps) {
  const [open, setOpen] = useState(false);
  const [input, setInput] = useState(value);
  const ref = useRef<HTMLDivElement>(null);

  const filtered = input.trim()
    ? accounts.filter((a) => a.name.toLowerCase().includes(input.toLowerCase()))
    : accounts;

  const handleSelect = useCallback(
    (name: string) => {
      setInput(name);
      onChange(name);
      setOpen(false);
    },
    [onChange],
  );

  const handleBlur = useCallback(() => {
    setTimeout(() => setOpen(false), 200);
    if (input.trim() && !accounts.find((a) => a.name === input.trim())) {
      onChange(input.trim());
    }
  }, [input, accounts, onChange]);

  useEffect(() => {
    setInput(value);
  }, [value]);

  return (
    <div ref={ref} className={`relative ${className}`}>
      <input
        type="text"
        value={input}
        onChange={(e) => {
          setInput(e.target.value);
          setOpen(true);
        }}
        onFocus={() => setOpen(true)}
        onBlur={handleBlur}
        placeholder="Type account name…"
        className="border rounded px-2 py-1 w-full"
      />
      {open && input.trim() && (
        <ul className="absolute z-10 bg-white border rounded mt-1 w-full max-h-40 overflow-auto shadow">
          {filtered.map((a) => (
            <li
              key={a.id}
              onMouseDown={() => handleSelect(a.name)}
              className="px-2 py-1 cursor-pointer hover:bg-gray-100"
            >
              {a.name}
            </li>
          ))}
          {input.trim() && !accounts.find((a) => a.name === input.trim()) && (
            <li className="px-2 py-1 text-gray-400 italic">
              Create "{input.trim()}"
            </li>
          )}
        </ul>
      )}
    </div>
  );
}
