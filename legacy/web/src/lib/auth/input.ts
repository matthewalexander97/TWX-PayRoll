export type LoginInput = { email: string; password: string; destination: string };
export type AuthState = { error?: string; email?: string };

const destinations = new Set(["/payroll", "/settlements", "/directory"]);

export function safeDestination(value: unknown): string {
  return typeof value === "string" && destinations.has(value) ? value : "/payroll";
}

export function parseLoginInput(form: FormData): LoginInput | null {
  const email = form.get("email");
  const password = form.get("password");
  if (typeof email !== "string" || typeof password !== "string") return null;
  const normalizedEmail = email.trim();
  if (normalizedEmail.length > 254 || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(normalizedEmail)) return null;
  if (password.length < 1 || password.length > 1024) return null;
  return { email: normalizedEmail, password, destination: safeDestination(form.get("next")) };
}
