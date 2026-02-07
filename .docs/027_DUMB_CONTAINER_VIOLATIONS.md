# 027 — Dumb Container Violations

Identified Feb 2025. Two places where parent components have centralized knowledge of all their children's types, violating the "dumb container" principle.

## The Principle

A container (like a browser tab bar) should not know how to render every possible child. A browser doesn't import every website — it loads content from a URL. Containers should render what they're given, not switch on a type to decide what to render.

The symptom: adding a new type requires modifying the container. The fix: children register themselves (or carry their own component), so the container just renders `tab.component` or looks up from a registry.

## Violation 1: WorkspacePanel.tsx (lines 71-89)

`TabContent` is a switch statement that imports and dispatches to all 8 tab content components:

```tsx
function TabContent({ tab }: { tab: WorkspaceTab }) {
  switch (tab.type) {
    case 'terminal':
      return <TerminalTabContent tab={tab} />
    case 'braindump-form':
      return <BraindumpFormTabContent tab={tab} />
    case 'task-form':
      return <TaskFormTabContent tab={tab} />
    // ... 5 more cases
  }
}
```

Lines 7-12 import every sibling content component. Adding a new tab type (e.g. `settings-form`) requires editing this file.

## Violation 2: App.tsx (lines 66-74)

Same pattern, conditional rendering form. The page "router" hardcodes all page components:

```tsx
<div className={currentPage === 'tasks' ? 'h-full' : 'hidden'}>
  <TasksPage />
</div>
<div className={currentPage === 'features' ? 'h-full' : 'hidden'}>
  <FeaturesPage />
</div>
<div className={currentPage === 'disciplines' ? 'h-full' : 'hidden'}>
  <DisciplinesPage />
</div>
```

All three page components are imported at the top of `App.tsx`. Adding a new page means editing the container.

## Fix Direction

Both share the same solution: the tab/page definition should carry (or register) its own component. The container renders it without knowing what it is. Options:

1. **Component on the tab object**: `WorkspaceTab` includes a `component` field set at `openTab()` call sites. Container renders `<tab.component />`.
2. **Registry pattern**: A `tabRegistry` maps type strings to lazy-loaded components. The container does `const C = tabRegistry[tab.type]; return <C tab={tab} />`. Registration happens at the content component's definition site, not the container.

Either way, `WorkspacePanel` and `App` stop importing content components and stop switching on types.
