import { clsx, type ClassValue } from 'clsx';
import { extendTailwindMerge } from 'tailwind-merge';

// Teach tailwind-merge Blotter's custom type scale so `text-sm` and
// `text-muted` aren't treated as the same (colour) group.
const twMerge = extendTailwindMerge({
  extend: {
    theme: {
      text: ['xs', 'sm', 'base', 'md', 'lg', 'xl', 'display'],
    },
  },
});

/** Joins class names, letting later Tailwind classes override earlier ones. */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
