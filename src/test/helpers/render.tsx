// ── Custom render helper ──────────────────────────────────────────────────────
// A thin wrapper around Testing Library's render() that can grow to include
// app-level providers (theme, router, etc.) as the app evolves.
// Prefer this over importing render directly so future context additions
// take effect everywhere automatically.

import { render } from "@testing-library/react";
import type { RenderOptions, RenderResult } from "@testing-library/react";
import type { ReactElement } from "react";

export { screen, fireEvent, waitFor, act } from "@testing-library/react";

export function renderWithProviders(
  ui: ReactElement,
  options?: Omit<RenderOptions, "wrapper">
): RenderResult {
  return render(ui, options);
}
