import js from "@eslint/js";
import ts from "typescript-eslint";
import hooks from "eslint-plugin-react-hooks";
export default ts.config(
  {
    ignores: ["legacy/**", "work/**", "dist/**", "src-tauri/**", "outputs/**"],
  },
  js.configs.recommended,
  ...ts.configs.recommended,
  {
    files: ["**/*.tsx"],
    plugins: { "react-hooks": hooks },
    rules: hooks.configs.recommended.rules,
  },
);
