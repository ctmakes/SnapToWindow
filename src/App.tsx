import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Shortcut {
  name: string;
  shortcut: string;
  action: string;
}

const shortcuts: Shortcut[] = [
  { name: "Left Half", shortcut: "⌃⌥←", action: "left_half" },
  { name: "Right Half", shortcut: "⌃⌥→", action: "right_half" },
  { name: "Top Half", shortcut: "⌃⌥↑", action: "top_half" },
  { name: "Bottom Half", shortcut: "⌃⌥↓", action: "bottom_half" },
  { name: "Top Left", shortcut: "⌃⌥U", action: "top_left" },
  { name: "Top Right", shortcut: "⌃⌥I", action: "top_right" },
  { name: "Bottom Left", shortcut: "⌃⌥J", action: "bottom_left" },
  { name: "Bottom Right", shortcut: "⌃⌥K", action: "bottom_right" },
  { name: "Maximize", shortcut: "⌃⌥↵", action: "maximize" },
  { name: "Center", shortcut: "⌃⌥C", action: "center" },
  { name: "Left Third", shortcut: "—", action: "left_third" },
  { name: "Center Third", shortcut: "—", action: "center_third" },
  { name: "Right Third", shortcut: "—", action: "right_third" },
  { name: "Left Two Thirds", shortcut: "—", action: "left_two_thirds" },
  { name: "Right Two Thirds", shortcut: "—", action: "right_two_thirds" },
];

function App() {
  const [accessibilityEnabled, setAccessibilityEnabled] = useState<boolean | null>(null);
  const [checking, setChecking] = useState(false);

  const checkAccessibility = async () => {
    setChecking(true);
    try {
      const enabled = await invoke<boolean>("check_accessibility");
      setAccessibilityEnabled(enabled);
    } catch (e) {
      console.error("Failed to check accessibility:", e);
      setAccessibilityEnabled(false);
    }
    setChecking(false);
  };

  const openAccessibilitySettings = async () => {
    try {
      await invoke("open_accessibility_settings");
      // Check again after a delay
      setTimeout(checkAccessibility, 1000);
    } catch (e) {
      console.error("Failed to open settings:", e);
    }
  };

  useEffect(() => {
    checkAccessibility();
    // Recheck when window becomes visible
    const interval = setInterval(checkAccessibility, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen bg-gray-900 text-white p-6">
      <h1 className="text-2xl font-bold mb-6">SnapToWindow</h1>

      {/* Accessibility Warning */}
      {accessibilityEnabled === false && (
        <div className="mb-6 p-4 bg-yellow-900/50 border border-yellow-600 rounded-lg">
          <div className="flex items-start gap-3">
            <svg
              className="w-6 h-6 text-yellow-500 flex-shrink-0 mt-0.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
            <div className="flex-1">
              <h3 className="font-semibold text-yellow-500">
                Accessibility Permission Required
              </h3>
              <p className="text-sm text-yellow-200/80 mt-1">
                SnapToWindow needs accessibility permissions to move and resize windows.
                Please enable it in System Settings.
              </p>
              <button
                onClick={openAccessibilitySettings}
                className="mt-3 px-4 py-2 bg-yellow-600 hover:bg-yellow-500 text-white rounded-md text-sm font-medium transition-colors"
              >
                Open Accessibility Settings
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Accessibility OK */}
      {accessibilityEnabled === true && (
        <div className="mb-6 p-4 bg-green-900/30 border border-green-700 rounded-lg">
          <div className="flex items-center gap-3">
            <svg
              className="w-6 h-6 text-green-500"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M5 13l4 4L19 7"
              />
            </svg>
            <span className="text-green-400">Accessibility permissions enabled</span>
          </div>
        </div>
      )}

      {/* Loading state */}
      {accessibilityEnabled === null && (
        <div className="mb-6 p-4 bg-gray-800 rounded-lg">
          <div className="flex items-center gap-3">
            <div className="w-5 h-5 border-2 border-gray-500 border-t-white rounded-full animate-spin" />
            <span className="text-gray-400">Checking permissions...</span>
          </div>
        </div>
      )}

      {/* Keyboard Shortcuts */}
      <div className="bg-gray-800 rounded-lg overflow-hidden">
        <div className="px-4 py-3 bg-gray-750 border-b border-gray-700">
          <h2 className="font-semibold text-gray-200">Keyboard Shortcuts</h2>
        </div>
        <div className="divide-y divide-gray-700">
          {shortcuts.map((shortcut) => (
            <div
              key={shortcut.action}
              className="px-4 py-3 flex justify-between items-center hover:bg-gray-750/50"
            >
              <span className="text-gray-300">{shortcut.name}</span>
              <span className="text-gray-500 font-mono text-sm bg-gray-700 px-2 py-1 rounded">
                {shortcut.shortcut}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Footer */}
      <div className="mt-6 text-center text-gray-500 text-sm">
        <p>SnapToWindow v0.1.0</p>
        <p className="mt-1">
          Inspired by{" "}
          <a
            href="https://rectangleapp.com"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:underline"
          >
            Rectangle
          </a>
        </p>
      </div>
    </div>
  );
}

export default App;
