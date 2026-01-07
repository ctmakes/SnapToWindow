import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function Logo() {
  return (
    <svg viewBox="0 0 100 22" className="h-6" fill="none" xmlns="http://www.w3.org/2000/svg">
      {/* Text: Snap */}
      <text x="0" y="16" fontFamily="system-ui, -apple-system, sans-serif" fontSize="16" fontWeight="600" fill="currentColor" className="text-white">
        Snap
      </text>
      {/* Text: To */}
      <text x="38" y="16" fontFamily="system-ui, -apple-system, sans-serif" fontSize="16" fontWeight="400" fill="currentColor" className="text-gray-400">
        To
      </text>
      {/* Window icon */}
      <g transform="translate(58, 4)">
        <rect x="0" y="0" width="20" height="14" rx="2" stroke="currentColor" strokeWidth="1.5" className="text-gray-400" />
        <rect x="1" y="1" width="8" height="12" rx="1" fill="currentColor" className="text-blue-500" />
        <circle cx="16" cy="3.5" r="1.2" fill="currentColor" className="text-gray-500" />
      </g>
    </svg>
  );
}

type SnapPosition =
  | "left_half"
  | "right_half"
  | "top_half"
  | "bottom_half"
  | "top_left"
  | "top_right"
  | "bottom_left"
  | "bottom_right"
  | "maximize"
  | "center"
  | "left_third"
  | "center_third"
  | "right_third"
  | "left_two_thirds"
  | "right_two_thirds";

interface ShortcutItem {
  name: string;
  shortcut: string;
  action: SnapPosition;
}

function SnapPreview({ position }: { position: SnapPosition }) {
  const getPreviewStyle = (): string => {
    const base = "absolute bg-blue-500 rounded-[1px]";
    switch (position) {
      case "left_half":
        return `${base} left-0 top-0 w-1/2 h-full`;
      case "right_half":
        return `${base} right-0 top-0 w-1/2 h-full`;
      case "top_half":
        return `${base} left-0 top-0 w-full h-1/2`;
      case "bottom_half":
        return `${base} left-0 bottom-0 w-full h-1/2`;
      case "top_left":
        return `${base} left-0 top-0 w-1/2 h-1/2`;
      case "top_right":
        return `${base} right-0 top-0 w-1/2 h-1/2`;
      case "bottom_left":
        return `${base} left-0 bottom-0 w-1/2 h-1/2`;
      case "bottom_right":
        return `${base} right-0 bottom-0 w-1/2 h-1/2`;
      case "maximize":
        return `${base} inset-0`;
      case "center":
        return `${base} left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-2/3 h-2/3`;
      case "left_third":
        return `${base} left-0 top-0 w-1/3 h-full`;
      case "center_third":
        return `${base} left-1/3 top-0 w-1/3 h-full`;
      case "right_third":
        return `${base} right-0 top-0 w-1/3 h-full`;
      case "left_two_thirds":
        return `${base} left-0 top-0 w-2/3 h-full`;
      case "right_two_thirds":
        return `${base} right-0 top-0 w-2/3 h-full`;
      default:
        return base;
    }
  };

  return (
    <div className="w-10 h-7 bg-gray-700 rounded border border-gray-600 relative overflow-hidden flex-shrink-0">
      <div className={getPreviewStyle()} />
    </div>
  );
}

function ShortcutRow({ item }: { item: ShortcutItem }) {
  return (
    <div className="flex items-center gap-2 py-1.5">
      <SnapPreview position={item.action} />
      <span className="text-sm text-gray-200 flex-1">{item.name}</span>
      <span className="text-xs text-gray-500 font-mono">{item.shortcut}</span>
    </div>
  );
}

function ShortcutColumn({ title, items }: { title: string; items: ShortcutItem[] }) {
  return (
    <div>
      <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">
        {title}
      </h3>
      <div className="flex flex-col">
        {items.map((item) => (
          <ShortcutRow key={item.action} item={item} />
        ))}
      </div>
    </div>
  );
}

const halves: ShortcutItem[] = [
  { name: "Left Half", shortcut: "⌃⌥←", action: "left_half" },
  { name: "Right Half", shortcut: "⌃⌥→", action: "right_half" },
  { name: "Top Half", shortcut: "⌃⌥↑", action: "top_half" },
  { name: "Bottom Half", shortcut: "⌃⌥↓", action: "bottom_half" },
];

const quarters: ShortcutItem[] = [
  { name: "Top Left", shortcut: "⌃⌥U", action: "top_left" },
  { name: "Top Right", shortcut: "⌃⌥I", action: "top_right" },
  { name: "Bottom Left", shortcut: "⌃⌥J", action: "bottom_left" },
  { name: "Bottom Right", shortcut: "⌃⌥K", action: "bottom_right" },
  { name: "Center", shortcut: "⌃⌥C", action: "center" },
  { name: "Maximize", shortcut: "⌃⌥↵", action: "maximize" },
];

const thirds: ShortcutItem[] = [
  { name: "Left Third", shortcut: "⌃⌥D", action: "left_third" },
  { name: "Center Third", shortcut: "⌃⌥F", action: "center_third" },
  { name: "Right Third", shortcut: "⌃⌥G", action: "right_third" },
  { name: "Left ⅔", shortcut: "⌃⌥E", action: "left_two_thirds" },
  { name: "Right ⅔", shortcut: "⌃⌥R", action: "right_two_thirds" },
];

function App() {
  const [accessibilityEnabled, setAccessibilityEnabled] = useState<boolean | null>(null);

  const checkAccessibility = async () => {
    try {
      const enabled = await invoke<boolean>("check_accessibility");
      setAccessibilityEnabled(enabled);
    } catch (e) {
      console.error("Failed to check accessibility:", e);
      setAccessibilityEnabled(false);
    }
  };

  const openAccessibilitySettings = async () => {
    try {
      await invoke("open_accessibility_settings");
      setTimeout(checkAccessibility, 1000);
    } catch (e) {
      console.error("Failed to open settings:", e);
    }
  };

  useEffect(() => {
    checkAccessibility();
    const interval = setInterval(checkAccessibility, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen bg-gray-900 text-white p-4">
      {/* Header */}
      <div className="flex items-center justify-center mb-4 relative">
        <Logo />
        {accessibilityEnabled === true && (
          <div className="absolute right-0 flex items-center gap-1.5 text-green-400 text-xs">
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            <span>Ready</span>
          </div>
        )}
      </div>

      {/* Accessibility Warning */}
      {accessibilityEnabled === false && (
        <div className="mb-4 p-3 bg-yellow-900/50 border border-yellow-600 rounded-lg">
          <div className="flex items-center gap-2">
            <svg className="w-4 h-4 text-yellow-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <div className="flex-1">
              <h3 className="font-medium text-yellow-500 text-sm">Accessibility Required</h3>
              <p className="text-xs text-yellow-200/80">Enable to move and resize windows.</p>
            </div>
            <button
              onClick={openAccessibilitySettings}
              className="px-2.5 py-1 bg-yellow-600 hover:bg-yellow-500 text-white rounded text-xs font-medium transition-colors flex-shrink-0"
            >
              Open Settings
            </button>
          </div>
        </div>
      )}

      {/* Loading state */}
      {accessibilityEnabled === null && (
        <div className="mb-4 p-2 bg-gray-800 rounded-lg">
          <div className="flex items-center gap-2">
            <div className="w-3.5 h-3.5 border-2 border-gray-500 border-t-white rounded-full animate-spin" />
            <span className="text-gray-400 text-xs">Checking permissions...</span>
          </div>
        </div>
      )}

      {/* 3-Column Shortcuts Grid */}
      <div className="grid grid-cols-3 gap-6">
        <ShortcutColumn title="Halves" items={halves} />
        <ShortcutColumn title="Quarters" items={quarters} />
        <ShortcutColumn title="Thirds" items={thirds} />
      </div>

      {/* Footer */}
      <div className="mt-4 pt-3 border-t border-gray-800 text-center text-gray-500 text-xs">
        v0.1.0 · Made with ♥ by{" "}
        <a href="https://ctmakes.com" target="_blank" rel="noopener noreferrer" className="text-blue-400 hover:underline">
          ctmakes
        </a>
      </div>
    </div>
  );
}

export default App;
