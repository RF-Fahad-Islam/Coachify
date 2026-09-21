import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { AuthProvider, useAuth } from "./auth-context";

function TestAuthConsumer() {
  const { currentUser, isOwner, hasPermission, isIdleLocked } = useAuth();
  return (
    <div data-testid="auth-consumer">
      <span data-testid="user">{currentUser?.display_name || "none"}</span>
      <span data-testid="is-owner">{isOwner() ? "yes" : "no"}</span>
      <span data-testid="has-manage-users">
        {hasPermission("users.manage") ? "yes" : "no"}
      </span>
      <span data-testid="locked">{isIdleLocked ? "yes" : "no"}</span>
    </div>
  );
}

describe("AuthContext and useAuth hook", () => {
  it("provides fallback auth context outside provider without throwing", () => {
    const html = renderToString(<TestAuthConsumer />);
    expect(html).toContain("Rohan Mehta");
    expect(html).toContain("locked");
  });

  it("renders inside AuthProvider cleanly", () => {
    const html = renderToString(
      <AuthProvider>
        <TestAuthConsumer />
      </AuthProvider>
    );
    expect(html).toContain('data-testid="auth-consumer"');
  });
});
