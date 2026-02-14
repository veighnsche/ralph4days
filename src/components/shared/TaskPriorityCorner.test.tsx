import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { TaskPriorityCorner } from './TaskPriorityCorner'

describe('TaskPriorityCorner', () => {
  it('does not render when priority is unset and showUnset=false', () => {
    const { container } = render(<TaskPriorityCorner priority={undefined} />)
    expect(container).toBeEmptyDOMElement()
  })

  it('renders a clickable button when onClick is provided (unset priority)', () => {
    const onClick = vi.fn()
    render(<TaskPriorityCorner priority={undefined} showUnset onClick={onClick} />)

    const button = screen.getByRole('button', { name: 'Cycle priority' })
    fireEvent.click(button)
    expect(onClick).toHaveBeenCalledTimes(1)
  })
})
