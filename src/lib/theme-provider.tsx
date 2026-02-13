import { ThemeProvider as NextThemesProvider, useTheme as useNextTheme } from 'next-themes'

export type Theme = 'dark' | 'light' | 'system'

type ThemeProviderProps = {
  children: React.ReactNode
  defaultTheme?: Theme
  storageKey?: string
}

type ThemeProviderState = {
  theme: Theme | undefined
  setTheme: (theme: Theme) => void
}

export function ThemeProvider({ children, defaultTheme = 'dark', storageKey = 'ralph-ui-theme' }: ThemeProviderProps) {
  return (
    <NextThemesProvider
      attribute="class"
      defaultTheme={defaultTheme}
      storageKey={storageKey}
      enableSystem
      disableTransitionOnChange>
      {children}
    </NextThemesProvider>
  )
}

export function useTheme(): ThemeProviderState {
  const { theme, setTheme } = useNextTheme()

  return {
    theme: theme as Theme | undefined,
    setTheme: (nextTheme: Theme) => setTheme(nextTheme)
  }
}
