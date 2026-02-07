import { Circle, icons, type LucideIcon } from 'lucide-react'

/**
 * MASSIVE alias map for AI-hallucinated icon names → actual Lucide icons
 *
 * This is NOT for human search - it's for AI agents writing YAML files.
 * When an AI writes icon: "wiring", we map it to "Cable".
 *
 * Think: What would an AI write for common disciplines/concepts?
 */
const AI_ICON_ALIASES: Record<string, string> = {
  // Wiring/Networking/Connectivity
  wiring: 'Cable',
  wire: 'Cable',
  cables: 'Cable',
  networking: 'Network',
  network: 'Network',
  connectivity: 'Cable',
  connection: 'Link',
  connected: 'Link',
  ethernet: 'Cable',

  // Frontend/UI
  frontend: 'Monitor',
  'front-end': 'Monitor',
  ui: 'Monitor',
  interface: 'LayoutDashboard',
  client: 'Monitor',
  web: 'Globe',
  website: 'Globe',
  browser: 'Globe',
  display: 'Monitor',
  screen: 'Monitor',
  view: 'Eye',

  // Backend/Server
  backend: 'Server',
  'back-end': 'Server',
  server: 'Server',
  api: 'Plug',
  service: 'Server',
  microservice: 'Boxes',
  services: 'Boxes',

  // Database/Storage
  database: 'Database',
  db: 'Database',
  data: 'Database',
  storage: 'HardDrive',
  cache: 'Database',
  persistence: 'Save',

  // Testing/QA
  testing: 'FlaskConical',
  test: 'FlaskConical',
  qa: 'CircleCheck',
  quality: 'Award',
  validation: 'SquareCheck',
  verification: 'CircleCheck',

  // Design/UX
  design: 'Palette',
  ux: 'Pencil',
  graphics: 'Palette',
  art: 'Palette',
  style: 'Palette',
  theme: 'Palette',
  branding: 'Palette',

  // Security/Auth
  security: 'Shield',
  auth: 'Lock',
  authentication: 'Lock',
  authorization: 'Key',
  permissions: 'Key',
  encryption: 'Lock',
  crypto: 'Lock',

  // DevOps/Deployment
  devops: 'Cloud',
  deployment: 'Rocket',
  deploy: 'Rocket',
  cicd: 'GitBranch',
  'ci/cd': 'GitBranch',
  pipeline: 'GitBranch',
  automation: 'Zap',

  // Infrastructure/Cloud
  infrastructure: 'Server',
  infra: 'Server',
  cloud: 'Cloud',
  hosting: 'Cloud',
  containers: 'Box',
  container: 'Box',
  docker: 'Box',
  kubernetes: 'Boxes',
  k8s: 'Boxes',

  // Monitoring/Observability
  monitoring: 'Eye',
  monitor: 'Eye',
  observability: 'Eye',
  logs: 'FileText',
  logging: 'FileText',
  metrics: 'ChartLine',
  analytics: 'ChartBar',
  tracing: 'GitBranch',

  // Documentation
  docs: 'BookOpen',
  documentation: 'BookOpen',
  readme: 'FileText',
  wiki: 'BookOpen',
  manual: 'BookOpen',
  guide: 'BookOpen',

  // Communication/Marketing
  marketing: 'Megaphone',
  communication: 'MessageCircle',
  messaging: 'MessageSquare',
  chat: 'MessageCircle',
  email: 'Mail',
  notifications: 'Bell',
  alerts: 'CircleAlert',

  // Development/Code
  development: 'Code',
  dev: 'Code',
  coding: 'Code',
  programming: 'Code',
  code: 'Code',
  terminal: 'Terminal',
  console: 'Terminal',
  shell: 'Terminal',
  cli: 'Terminal',

  // Version Control
  git: 'GitBranch',
  vcs: 'GitBranch',
  version: 'GitBranch',
  repository: 'FolderGit',
  repo: 'FolderGit',

  // Build/Compile
  build: 'Hammer',
  compile: 'Hammer',
  bundler: 'Package',
  webpack: 'Package',

  // Navigation/Menu
  menu: 'Menu',
  navigation: 'Menu',
  nav: 'Menu',
  hamburger: 'Menu',
  sidebar: 'PanelLeft',

  // Actions (common verbs AI uses)
  add: 'Plus',
  create: 'Plus',
  new: 'Plus',
  edit: 'Pencil',
  modify: 'Pencil',
  update: 'RefreshCw',
  delete: 'Trash2',
  remove: 'Trash2',
  trash: 'Trash2',
  save: 'Save',
  download: 'Download',
  upload: 'Upload',
  send: 'Send',
  submit: 'Send',
  search: 'Search',
  find: 'Search',
  filter: 'ListFilter',
  sort: 'ArrowUpDown',

  // Settings/Config
  settings: 'Settings',
  config: 'Settings',
  configuration: 'Settings',
  preferences: 'Settings',
  options: 'Settings',
  gear: 'Settings',
  cog: 'Settings',

  // Files/Folders
  file: 'File',
  files: 'Files',
  folder: 'Folder',
  directory: 'Folder',
  document: 'FileText',
  documents: 'FileText',

  // Media
  image: 'Image',
  images: 'Image',
  photo: 'Image',
  picture: 'Image',
  video: 'Video',
  audio: 'Music',
  media: 'Image',

  // User/Account
  user: 'User',
  users: 'Users',
  account: 'User',
  profile: 'User',
  avatar: 'User',
  person: 'User',

  // Business/Commerce
  business: 'Briefcase',
  commerce: 'ShoppingCart',
  shop: 'ShoppingCart',
  store: 'ShoppingCart',
  cart: 'ShoppingCart',
  payment: 'CreditCard',
  billing: 'DollarSign',
  invoice: 'Receipt',

  // Time/Schedule
  time: 'Clock',
  clock: 'Clock',
  schedule: 'Calendar',
  calendar: 'Calendar',
  date: 'Calendar',
  timer: 'Timer',

  // Status/State
  success: 'CircleCheck',
  error: 'CircleX',
  warning: 'TriangleAlert',
  info: 'Info',
  help: 'CircleQuestionMark',
  question: 'CircleQuestionMark',

  // Performance
  performance: 'Zap',
  speed: 'Zap',
  fast: 'Zap',
  optimization: 'Zap',
  optimize: 'Zap',

  // Mobile/Devices
  mobile: 'Smartphone',
  phone: 'Smartphone',
  tablet: 'Tablet',
  desktop: 'Monitor',
  laptop: 'Laptop',
  device: 'Smartphone',

  // Social/Sharing
  share: 'Share2',
  social: 'Share2',
  like: 'Heart',
  favorite: 'Heart',
  star: 'Star',

  // Location
  location: 'MapPin',
  map: 'Map',
  place: 'MapPin',
  gps: 'MapPin',

  // Power/Control
  power: 'Power',
  start: 'Play',
  stop: 'Square',
  pause: 'Pause',
  restart: 'RotateCcw',

  // Data Flow
  input: 'Download',
  output: 'Upload',
  import: 'Download',
  export: 'Upload',
  transfer: 'ArrowLeftRight',
  sync: 'RefreshCw',

  // Organization
  organization: 'Building',
  company: 'Building',
  team: 'Users',
  group: 'Users',

  // Learning/Education
  learning: 'GraduationCap',
  education: 'GraduationCap',
  course: 'BookOpen',
  tutorial: 'BookOpen',

  // Plugins/Extensions
  plugin: 'Plug',
  plugins: 'Plug',
  extension: 'Puzzle',
  addon: 'Puzzle',
  module: 'Box',

  // Reports/Analytics
  report: 'ChartBar',
  reports: 'ChartBar',
  dashboard: 'LayoutDashboard',
  chart: 'ChartBar',
  graph: 'ChartLine',

  // Tasks/Todos
  task: 'SquareCheck',
  tasks: 'ListChecks',
  todo: 'SquareCheck',
  checklist: 'ListChecks',

  // Accessibility
  accessibility: 'Accessibility',
  a11y: 'Accessibility',

  // Internationalization
  language: 'Languages',
  translate: 'Languages',
  translation: 'Languages',
  i18n: 'Languages',

  // Backup/Recovery
  backup: 'Archive',
  restore: 'RotateCcw',
  recovery: 'RotateCcw',

  // Events
  event: 'Zap',
  trigger: 'Zap',

  // Tags/Labels
  tag: 'Tag',
  tags: 'Tags',
  label: 'Tag',
  labels: 'Tags',

  // Print
  print: 'Printer',
  printer: 'Printer',

  // Copy/Paste
  copy: 'Copy',
  paste: 'Clipboard',
  clipboard: 'Clipboard',

  // Undo/Redo
  undo: 'Undo',
  redo: 'Redo',

  // Visibility
  visible: 'Eye',
  hidden: 'EyeOff',
  show: 'Eye',
  hide: 'EyeOff',

  // Lock/Unlock
  lock: 'Lock',
  locked: 'Lock',
  unlock: 'LockOpen',
  unlocked: 'LockOpen',

  // Public/Private
  public: 'Globe',
  private: 'Lock',

  // Link/Unlink
  link: 'Link',
  unlink: 'Unlink',
  url: 'Link',

  // Expand/Collapse
  expand: 'ChevronDown',
  collapse: 'ChevronUp',

  // Refresh/Reload
  refresh: 'RefreshCw',
  reload: 'RefreshCw',

  // Close/Exit
  close: 'X',
  exit: 'X',
  cancel: 'X',

  // Home
  home: 'House',
  house: 'House',

  // Maximize/Minimize
  maximize: 'Maximize',
  minimize: 'Minimize',
  fullscreen: 'Maximize2',

  // Volume/Audio
  volume: 'Volume2',
  mute: 'VolumeX',
  sound: 'Volume2',

  // Brightness
  brightness: 'Sun',
  dark: 'Moon',
  light: 'Sun',

  // Battery
  battery: 'Battery',
  charging: 'BatteryCharging',

  // WiFi/Signal
  wifi: 'Wifi',
  signal: 'Signal',

  // Bluetooth
  bluetooth: 'Bluetooth',

  // USB
  usb: 'Usb',

  // Camera
  camera: 'Camera',
  webcam: 'Camera',

  // Microphone
  microphone: 'Mic',
  mic: 'Mic',

  // External/Internal
  external: 'ExternalLink',
  internal: 'House',

  // API/Integration specific
  webhook: 'Webhook',
  rest: 'Plug',
  graphql: 'Network',
  grpc: 'Network',
  soap: 'Cloud',

  // Specific tech terms AI might use
  lambda: 'Zap',
  function: 'Code',
  serverless: 'Cloud',
  edge: 'Zap',
  cdn: 'Globe',
  dns: 'Globe',
  ssl: 'Lock',
  firewall: 'Shield',
  loadbalancer: 'Scale',
  queue: 'List',
  stream: 'Radio',
  batch: 'Layers',
  cron: 'Clock',

  // Deployment environments
  production: 'Rocket',
  staging: 'FlaskConical',
  local: 'House',

  // Branches/Environments
  main: 'GitBranch',
  master: 'GitBranch',
  feature: 'GitBranch',
  bugfix: 'Bug',
  hotfix: 'Flame',

  // Issue tracking
  bug: 'Bug',
  issue: 'CircleAlert',
  ticket: 'Ticket',

  // Workflow
  workflow: 'GitBranch',
  process: 'GitBranch',
  flow: 'GitBranch'
}

/**
 * Resolve icon name for AI-generated YAML files.
 *
 * This is optimized for AI agents, not humans:
 * - AI writes "wiring" → get "Cable"
 * - AI writes "networking" → get "Network"
 * - AI writes "deployment" → get "Rocket"
 *
 * Strategy:
 * 1. Exact match (PascalCase)
 * 2. Lowercase lookup in alias map (500+ entries)
 * 3. Fallback to Circle
 */
export function resolveIcon(name: string): LucideIcon {
  if (!name) {
    console.warn('resolveIcon called with empty name')
    return Circle
  }

  // 1. Try exact match (case-sensitive, PascalCase)
  if (icons[name as keyof typeof icons]) {
    return icons[name as keyof typeof icons] as LucideIcon
  }

  // 2. AI alias lookup (lowercase)
  const normalized = name.toLowerCase().trim()
  const aliasMatch = AI_ICON_ALIASES[normalized]

  if (aliasMatch && icons[aliasMatch as keyof typeof icons]) {
    console.log(`✅ Icon resolved: "${name}" → "${aliasMatch}" (alias)`)
    return icons[aliasMatch as keyof typeof icons] as LucideIcon
  }

  // 3. Fallback
  console.warn(`❌ Icon not found for: "${name}", falling back to Circle`)
  return Circle
}

/**
 * VALIDATION: Run this on startup in dev mode to catch hallucinated icon names
 */
if (import.meta.env.DEV) {
  const invalidIcons: string[] = []

  for (const [alias, iconName] of Object.entries(AI_ICON_ALIASES)) {
    if (!icons[iconName as keyof typeof icons]) {
      invalidIcons.push(`"${alias}" → "${iconName}" (DOES NOT EXIST)`)
    }
  }

  if (invalidIcons.length > 0) {
    console.error('❌ Icon Registry Validation Failed!')
    console.error(`Found ${invalidIcons.length} hallucinated icon names:\n`)
    for (const err of invalidIcons) {
      console.error(`  ${err}`)
    }
    throw new Error(
      `Icon registry contains ${invalidIcons.length} non-existent icons. Fix AI_ICON_ALIASES in iconRegistry.ts`
    )
  }
}
