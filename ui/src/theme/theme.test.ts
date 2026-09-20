import { describe, it, expect } from "vitest";
import { DEFAULT_THEME_PREFERENCES } from "./theme-provider";

describe("Theme system configuration", () => {
  it("defaults to Sunset Orange palette matching EduManage reference", () => {
    expect(DEFAULT_THEME_PREFERENCES.palette).toBe("orange");
  });

  it("defaults to OS native system font and spacious density", () => {
    expect(DEFAULT_THEME_PREFERENCES.fontFamily).toBe("system");
    expect(DEFAULT_THEME_PREFERENCES.density).toBe("spacious");
  });

  it("defaults to light mode and 100% UI scale", () => {
    expect(DEFAULT_THEME_PREFERENCES.mode).toBe("light");
    expect(DEFAULT_THEME_PREFERENCES.uiScale).toBe(100);
    expect(DEFAULT_THEME_PREFERENCES.radius).toBe("rounded");
  });
});
