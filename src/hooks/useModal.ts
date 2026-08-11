import { useState, useCallback, useEffect } from 'react'

interface UseModalReturn {
  isOpen: boolean
  open: () => void
  close: () => void
  toggle: () => void
}

export function useModal(initialState = false): UseModalReturn {
  const [isOpen, setIsOpen] = useState(initialState)

  const open = useCallback(() => setIsOpen(true), [])
  const close = useCallback(() => setIsOpen(false), [])
  const toggle = useCallback(() => setIsOpen(prev => !prev), [])

  return { isOpen, open, close, toggle }
}

interface UseModalWithKeyboardOptions {
  escapeToClose?: boolean
  onEscape?: () => void
}

export function useModalWithKeyboard(
  initialState = false,
  options: UseModalWithKeyboardOptions = {}
): UseModalReturn {
  const { escapeToClose = true, onEscape } = options
  const [isOpen, setIsOpen] = useState(initialState)

  const open = useCallback(() => setIsOpen(true), [])
  const close = useCallback(() => setIsOpen(false), [])
  const toggle = useCallback(() => setIsOpen(prev => !prev), [])

  useEffect(() => {
    if (!isOpen || !escapeToClose) return

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        if (onEscape) {
          onEscape()
        } else {
          close()
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, escapeToClose, onEscape, close])

  return { isOpen, open, close, toggle }
}