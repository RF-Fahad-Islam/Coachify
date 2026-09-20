import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { AuthProvider } from "@/contexts/auth-context";
import { IdleLockDialog } from "./idle-lock-dialog";
import { BootstrapOwnerDialog } from "./bootstrap-owner-dialog";
import { LoginDialog } from "./login-dialog";

describe("Auth Dialog Components", () => {
  it("renders without crashing inside AuthProvider", () => {
    const html = renderToString(
      <AuthProvider>
        <div>
          <IdleLockDialog />
          <BootstrapOwnerDialog />
          <LoginDialog />
        </div>
      </AuthProvider>
    );
    expect(html).toBeDefined();
  });
});
