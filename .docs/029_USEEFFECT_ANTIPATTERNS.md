# useEffect Antipatterns Reference

Comprehensive catalog of wrong ways to use `useEffect`, compiled from the React docs, community analysis, and production codebase reviews. Use this as a checklist when reviewing PRs or refactoring existing hooks.

## Category 1: You Don't Need an Effect At All

### Derived/Computed State

The #1 misuse. Using `useEffect` to compute something from existing props/state causes an unnecessary extra render cycle with a stale intermediate value.

```tsx
// WRONG: extra render with stale value, then re-render with correct value
const [fullName, setFullName] = useState('');
useEffect(() => {
  setFullName(firstName + ' ' + lastName);
}, [firstName, lastName]);

// RIGHT: just calculate it
const fullName = firstName + ' ' + lastName;
```

This applies to any filtering, mapping, or transformation of props/state. If `getFilteredTodos()` is expensive, use `useMemo` (or let React Compiler handle it), not `useEffect`.

### Resetting State on Prop Change

Using `useEffect` to clear state when a prop changes causes a render with stale state followed by a re-render with the reset state. Use a `key` to let React unmount/remount cleanly.

```tsx
// WRONG: renders once with stale comment, then again with empty
useEffect(() => { setComment(''); }, [userId]);

// RIGHT: React treats different keys as different component instances
<Profile userId={userId} key={userId} />
```

If you only need to reset *some* state, store the minimum identifier (e.g., `selectedId`) and derive the rest during render rather than imperatively resetting.

### Event-Specific Logic

If something happens because a user clicked a button, it belongs in the click handler, not an effect watching for the resulting state change.

```tsx
// WRONG: fires on page reload too, not just on user action
useEffect(() => {
  if (product.isInCart) showNotification(`Added ${product.name}!`);
}, [product]);

// RIGHT: in the event handler
function handleBuy() {
  addToCart(product);
  showNotification(`Added ${product.name}!`);
}
```

The test: "Would this code make sense if the user just *landed* on the page?" If no, it belongs in an event handler.

### Chains of Effects Triggering Each Other

Multiple effects that each `setState` to trigger the next one cause cascading re-renders (e.g., `setCard` -> effect sets `setGoldCardCount` -> effect sets `setRound` -> effect sets `setIsGameOver`). Calculate everything in the event handler or during render instead.

### Notifying Parent Components About State Changes

Using an effect to call `onChange(localState)` whenever local state changes. Call `onChange` in the same event handler where you set the state. React batches updates from different components in the same event handler into a single render pass.

```tsx
// WRONG: extra render cycle, onChange fires too late
useEffect(() => { onChange(isOn); }, [isOn, onChange]);

// RIGHT: both updates happen in the same event
function handleClick() {
  setIsOn(!isOn);
  onChange(!isOn);
}
```

### Passing Data to Parent

Child fetches data, then uses an effect to call `onFetched(data)`. This reverses React's data flow and makes bugs hard to trace. Lift the data fetching to the parent and pass data down as props.

### Sending POST Requests via State Triggers

Setting a `jsonToSubmit` state variable and watching it in an effect to fire a POST request. The form submission is a user event — put the `fetch` call directly in `handleSubmit`.

## Category 2: Dependency Array Disasters

### Missing Dependency Array Entirely

`useEffect(() => { ... })` with no array runs on **every single render**. Removing deps from the array makes the effect run *less* often, but removing the array entirely makes it run *always*. This asymmetry is the biggest footgun in the API.

### Missing Deps Causing Stale Closures

Forgetting to include a prop like `userId` in the deps array means the effect never re-runs when that prop changes, showing stale data forever.

```tsx
// WRONG: effect runs once, never refetches when userId changes
useEffect(() => { fetchUser(userId); }, []);

// RIGHT
useEffect(() => { fetchUser(userId); }, [userId]);
```

Enable `eslint-plugin-react-hooks` to catch these automatically.

### Unstable Object/Function References

Objects and functions created during render are new references every time. Putting them in deps causes infinite re-runs.

```tsx
// WRONG: `user` is a new object reference every render
const user = { userId: 123 };
useEffect(() => { fetchUser(user.userId); }, [user]); // runs every render

// RIGHT: use the primitive value directly
useEffect(() => { fetchUser(userId); }, [userId]);
```

For functions, either hoist them outside the component, move them inside the effect, or (if React Compiler isn't available) wrap with `useCallback`.

### Stale State Reads

Reading state right after calling `setState` inside an effect. State updates are async; the variable still holds the previous value on the next line.

```tsx
// WRONG: user is still null here
setUser(data);
console.log("Fetched user:", user); // null

// RIGHT: use the data directly, or react to the state change in a separate effect
setUser(data);
console.log("Fetched user:", data);
```

### Multiple Effects With the Same Dependency

Three separate `useEffect` calls all watching `[userId]`. Combine related logic into one effect. Keep effects separate only when they're logically unrelated (e.g., one for analytics, one for a WebSocket).

### Unnecessary Dependencies

Adding extra deps that don't affect the effect's logic. Since the component re-renders when props change anyway, derived values don't need effects at all.

## Category 3: Missing Cleanup

### Not Cancelling Fetch Requests

If the component unmounts mid-fetch, you get setState-on-unmounted-component warnings or stale data overwrites.

```tsx
useEffect(() => {
  const controller = new AbortController();
  fetch(url, { signal: controller.signal }).then(/* ... */);
  return () => controller.abort();
}, [url]);
```

### Race Conditions in Data Fetching

Typing "hello" fires fetches for "h", "he", "hel", "hell", "hello". Without cleanup, responses arrive out of order and the wrong result wins.

```tsx
useEffect(() => {
  let ignore = false;
  fetchResults(query).then(json => {
    if (!ignore) setResults(json);
  });
  return () => { ignore = true; };
}, [query]);
```

### Not Removing Event Listeners

Adding `window.addEventListener` without returning a cleanup that calls `removeEventListener`. Leaks listeners on every mount cycle.

```tsx
useEffect(() => {
  window.addEventListener('resize', onResize);
  return () => window.removeEventListener('resize', onResize);
}, []);
```

### Listener Re-registration on Deps Change

The deps array forces the entire listener to be unsubscribed and resubscribed whenever any dep changes. This can break event ordering (e.g., modal `Esc` handler vs page `Esc` handler depend on registration order) or cause expensive re-subscriptions in external systems.

## Category 4: Wrong Hook Entirely

### DOM Measurement Before Paint

`useEffect` runs **after** the browser paints. If you need to measure or mutate the DOM before the user sees it (tooltip positioning, scroll restoration), use `useLayoutEffect`. Using `useEffect` causes visible flicker.

### External Store Subscriptions

Manually subscribing to external stores (`navigator.onLine`, third-party state libs) in `useEffect`. Use `useSyncExternalStore` instead — it's purpose-built, avoids tearing, and handles server rendering.

### App-Level Initialization

Putting one-time setup (`initAnalytics()`, `checkAuthToken()`) in a component's `useEffect`. It runs twice in Strict Mode and on every remount. Use a module-level guard or run it outside React entirely.

```tsx
// WRONG: runs twice in dev, runs again on remount
useEffect(() => { initFacebookPixel(); }, []);

// RIGHT: module-level guard
let didInit = false;
function App() {
  useEffect(() => {
    if (!didInit) {
      didInit = true;
      initFacebookPixel();
    }
  }, []);
}
```

### useEffectEvent for Non-Reactive Logic (React 19.2+)

When you need to read the latest props/state inside an effect without adding them to the deps array (e.g., analytics tracking that reads current metadata but shouldn't re-fire when metadata changes), use `useEffectEvent` to extract the non-reactive part.

## Category 5: Architectural Misuse

### Treating useEffect as componentDidMount

They are not the same. Class lifecycle methods run synchronously before paint; `useEffect` runs asynchronously after paint. This timing difference matters for DOM mutations, measurements, and anything the user can see flicker.

### Triggering Re-renders During Animations

Using state updates on every animation frame runs the entire React render pipeline at 60fps. Use refs and imperative style updates instead.

```tsx
// WRONG: causes full React re-render 60 times per second
const [x, setX] = useState(0);
useEffect(() => {
  const id = requestAnimationFrame(() => setX(prev => prev + 1));
  return () => cancelAnimationFrame(id);
}, [x]);

// RIGHT: mutate the DOM directly via ref
const ref = useRef<HTMLDivElement>(null);
useEffect(() => {
  let frame: number;
  function animate() {
    if (ref.current) ref.current.style.transform = `translateX(${x}px)`;
    frame = requestAnimationFrame(animate);
  }
  frame = requestAnimationFrame(animate);
  return () => cancelAnimationFrame(frame);
}, []);
```

This also applies to drag handlers, `mousemove`, window resize, and scroll events. Perform state updates only at the start and end of the interaction, not on every frame.

## The Golden Rule

> Ask yourself: "Is there an external system I'm synchronizing with?" If no, you probably don't need `useEffect`.

Valid uses: data fetching with cleanup, subscriptions to external systems, DOM synchronization with non-React widgets, analytics on mount, third-party library integration.

## Sources

- https://react.dev/learn/you-might-not-need-an-effect
- https://blog.logrocket.com/15-common-useeffect-mistakes-react/
- https://alexkritchevsky.com/2023/04/25/react-mistakes.html
