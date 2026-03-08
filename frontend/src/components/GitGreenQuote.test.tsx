import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { GitGreenQuote } from './GitGreenQuote'
import type { Quote } from '@/lib/quotes'

const testQuote: Quote = {
  text: 'Test quote text here',
  author: 'Test Author',
}

describe('GitGreenQuote', () => {
  it('renders the quote text', () => {
    render(<GitGreenQuote quote={testQuote} />)
    expect(screen.getByText(/Test quote text here/)).toBeTruthy()
  })

  it('renders the author with an em-dash', () => {
    render(<GitGreenQuote quote={testQuote} />)
    expect(screen.getByText(/Test Author/)).toBeTruthy()
    // The rendered text should contain an em-dash before the author
    const authorEl = screen.getByText(/Test Author/)
    expect(authorEl.textContent).toContain('\u2014')
  })

  it('renders without a quote prop (uses weekly default)', () => {
    const { container } = render(<GitGreenQuote />)
    // Should render something (the weekly quote)
    const paragraphs = container.querySelectorAll('p')
    expect(paragraphs.length).toBe(2) // quote text + author
  })
})
