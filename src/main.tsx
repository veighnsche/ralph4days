import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import ReactDOM from 'react-dom/client'
import { ThemeProvider } from '@/lib/theme-provider'
import App from './App'

if (import.meta.env.DEV) {
  import('./lib/dev-bridge')
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      refetchOnWindowFocus: false,
      retry: 1
    }
  }
})

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <QueryClientProvider client={queryClient}>
    <ThemeProvider defaultTheme="dark">
      <App />
    </ThemeProvider>
  </QueryClientProvider>
)
