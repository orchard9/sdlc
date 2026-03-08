/** Static quote corpus for the git-green-quotes feature. */

export interface Quote {
  text: string;
  author: string;
}

/**
 * Curated quotes displayed when the git working tree is clean.
 * Add entries freely -- the weekly rotation adapts to any array length.
 */
export const QUOTES: Quote[] = [
  {
    text: "First, solve the problem. Then, write the code.",
    author: "John Johnson",
  },
  {
    text: "Simplicity is the soul of efficiency.",
    author: "Austin Freeman",
  },
  {
    text: "Make it work, make it right, make it fast.",
    author: "Kent Beck",
  },
  {
    text: "Programs must be written for people to read, and only incidentally for machines to execute.",
    author: "Harold Abelson",
  },
  {
    text: "The best code is no code at all.",
    author: "Jeff Atwood",
  },
  {
    text: "Controlling complexity is the essence of computer programming.",
    author: "Brian Kernighan",
  },
  {
    text: "Clean code always looks like it was written by someone who cares.",
    author: "Robert C. Martin",
  },
  {
    text: "Any fool can write code that a computer can understand. Good programmers write code that humans can understand.",
    author: "Martin Fowler",
  },
  {
    text: "The function of good software is to make the complex appear to be simple.",
    author: "Grady Booch",
  },
  {
    text: "Truth can only be found in one place: the code.",
    author: "Robert C. Martin",
  },
  {
    text: "Perfection is achieved not when there is nothing more to add, but when there is nothing left to take away.",
    author: "Antoine de Saint-Exupery",
  },
  {
    text: "Code is like humor. When you have to explain it, it's bad.",
    author: "Cory House",
  },
  {
    text: "Before software can be reusable it first has to be usable.",
    author: "Ralph Johnson",
  },
  {
    text: "Measuring programming progress by lines of code is like measuring aircraft building progress by weight.",
    author: "Bill Gates",
  },
  {
    text: "A ship in port is safe, but that's not what ships are built for.",
    author: "Grace Hopper",
  },
  {
    text: "Walking on water and developing software from a specification are easy if both are frozen.",
    author: "Edward Berard",
  },
];

/** Milliseconds in one week. */
const MS_PER_WEEK = 7 * 24 * 60 * 60 * 1000;

/**
 * Returns the quote for the current calendar week.
 *
 * Deterministic: every caller during the same 7-day window gets the same
 * quote. The window resets every {@link MS_PER_WEEK} milliseconds from the
 * Unix epoch.
 *
 * @param quotes - Override the default corpus (useful for testing).
 * @param now    - Override Date.now() (useful for testing).
 */
export function getWeeklyQuote(
  quotes: Quote[] = QUOTES,
  now: number = Date.now(),
): Quote {
  const weekIndex = Math.floor(now / MS_PER_WEEK);
  return quotes[weekIndex % quotes.length];
}
