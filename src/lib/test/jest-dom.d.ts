// Registers the jest-dom matcher types (toHaveTextContent, ...) for svelte-check;
// vitest-setup.ts sits outside the SvelteKit tsconfig's include paths.
import "@testing-library/jest-dom/vitest";
