/**
 * useModelThinkingPreferences - Hook to manage Claude model and thinking mode preferences
 *
 * Provides shared state management and localStorage persistence for model/thinking preferences.
 * Used across multiple components (braindump form, workspace panel, etc.)
 */

import { useEffect, useState } from "react";

const STORAGE_KEY_MODEL = "ralph.preferences.model";
const STORAGE_KEY_THINKING = "ralph.preferences.thinking";

const VALID_MODELS = ["haiku", "sonnet", "opus"] as const;
export type Model = (typeof VALID_MODELS)[number];

function isValidModel(value: string | null): value is Model {
  return VALID_MODELS.includes(value as Model);
}

export function useModelThinkingPreferences() {
  const [model, setModel] = useState<Model>(() => {
    const saved = localStorage.getItem(STORAGE_KEY_MODEL);
    return isValidModel(saved) ? saved : "sonnet";
  });

  const [thinking, setThinking] = useState(() => {
    const saved = localStorage.getItem(STORAGE_KEY_THINKING);
    return saved === "true";
  });

  // Persist model preference (validate before saving)
  useEffect(() => {
    if (isValidModel(model)) {
      localStorage.setItem(STORAGE_KEY_MODEL, model);
    }
  }, [model]);

  // Persist thinking preference
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY_THINKING, String(thinking));
  }, [thinking]);

  return {
    model,
    setModel,
    thinking,
    setThinking,
  };
}
