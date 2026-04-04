import eslint from "@eslint/js";
import html from "@html-eslint/eslint-plugin";
import htmlParser from "@html-eslint/parser";
import tseslint from "typescript-eslint";

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    ignores: ["dist/"],
  },
  {
    files: ["**/*.html"],
    plugins: { "@html-eslint": html },
    languageOptions: { parser: htmlParser },
    rules: {
      ...html.configs["flat/recommended"].rules,
      // Disable formatting rules handled by Prettier
      "@html-eslint/indent": "off",
      "@html-eslint/no-extra-spacing-attrs": "off",
      "@html-eslint/attrs-newline": "off",
      "@html-eslint/element-newline": "off",
      "@html-eslint/closing-bracket-newline": "off",
      // Allow self-closing tags for void elements
      "@html-eslint/require-closing-tags": ["error", { selfClosing: "always" }],
    },
  },
);
