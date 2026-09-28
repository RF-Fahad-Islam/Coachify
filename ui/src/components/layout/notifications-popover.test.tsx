import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { NotificationsPopover } from "./notifications-popover";

describe("NotificationsPopover component", () => {
  it("renders notification trigger button with unread count badge", () => {
    const html = renderToString(<NotificationsPopover onNavigateTab={() => {}} />);
    expect(html).toContain("Open notifications menu");
    // Initial 3 unread items badge
    expect(html).toContain("3");
    expect(html).toContain("bg-primary text-[10px]");
  });

  it("renders without error when onNavigateTab is not provided", () => {
    const html = renderToString(<NotificationsPopover />);
    expect(html).toContain("Open notifications menu");
  });
});
