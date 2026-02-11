# Blind Verb Simulations - Batch I (Iterations 1-60 Ralph Loop)

Simulations 21+ from Ralph loop validation: each iteration generates ONE blind simulation, validates friction against 8-verb set, loops until 100% clean mapping.

---

## Simulation #21 (Ralph Loop Iteration 1)

**Project:** Video transcription service (Python/Whisper)
**Feature:** speaker-diarization
**Discipline:** backend
**Task:** Implement speaker identification to separate dialogue by speaker in transcripts. Mark which speaker says what in the output JSON.
**Complication:** Prior task left incomplete code

### Walkthrough

I start by reading the existing Whisper integration. I find a `transcribe()` function that returns segments with timestamps and text. I search for diarization code. There's a file `speaker_identification.py` that's mostly empty — just import statements and a TODO comment saying "integrate pyannote.audio".

**I need to know: did the previous task ("Set up speaker diarization models") actually download and cache the pyannote models, or is that also a TODO?** I check the models directory. It's empty except for a `.gitkeep` file. So the prior task was marked done but left the infrastructure stub. I can't proceed without the model weights.

I search for documentation on how pyannote should be integrated. I find a requirements.txt that lists `pyannote.audio==2.1.0` but there's no initialization code. **Should I assume the model will be downloaded at runtime? Should I add a setup step? Should I cache it in a specific directory?** The task description doesn't clarify.

I start writing the diarization wrapper anyway. I'll use `pyannote.audio` to detect speaker changes, map speakers to IDs, and annotate the transcript. But **I'm making assumptions about the deployment model: will the model be cached before the service starts? Will it download on first request? This affects error handling and latency characteristics.**

I also realize: the transcription segments might not align perfectly with speaker changes. A sentence could span two speakers. **How do I handle partial speaker overlap?** Do I:
1. Split the segment at the speaker boundary
2. Assign the whole segment to the dominant speaker
3. Create overlapping segments
4. Flag it for manual review

The task acceptance criteria just say "mark which speaker says what" — that's ambiguous.

I implement option 2 (assign to dominant speaker) and add a confidence score. But **I'm leaving technical debt: if partial overlaps become a problem in production, someone will need to refactor this.**

### Friction Points Extracted

1. **ask** — "Did the prior task download models or not?" (blocked on infrastructure decision)
2. **flag** — Prior task marked done but left infrastructure stub
3. **learned** — Chose dominant-speaker heuristic without domain validation
4. **suggest** — Should document model caching strategy

### Validation

- `ask` ✓ (decision needed to proceed)
- `flag` ✓ (found incomplete prior work)
- `learned` ✓ (design decision with rationale)
- `suggest` ✓ (recommendation for future task)

**Result:** 4/4 friction points map to 8-verb set. Continue looping.

---

## Simulation #22 (Ralph Loop Iteration 2)

**Project:** Internal HR portal (Next.js/TypeScript)
**Feature:** time-off-management
**Discipline:** fullstack
**Task:** Build the leave request workflow: employees submit requests, managers approve/deny, HR gets audit trail. Must enforce company policies (max 30 days/year, blackout dates).
**Complication:** Task description contradicts codebase

### Walkthrough

I read the task. It says "enforce company policies" — max 30 days/year. I search the codebase for where policies are stored. I find a `policies.ts` file with hardcoded constants: `MAX_ANNUAL_LEAVE: 25`. **The task says 30, the code says 25. Which is correct?** I search for tickets or comments. Nothing.

I check git history. The policy was set to 25 three months ago. The task was written last week. Either the policy changed and the task wasn't updated, or the task writer got the number wrong. **I have no way to know which.**

I also search for blackout dates. The codebase doesn't have a blackout_dates table or configuration. **Do I create it? Should I assume blackout dates are managed elsewhere? The task says "enforce company policies" but doesn't specify where they come from.**

I also realize: "max 30 days/year" is ambiguous. Is it calendar year or fiscal year? Renewable or use-it-or-lose-it? Can employees carry over unused days? **The task doesn't say, and I need to pick.**

I decide: use the hardcoded 25 (trusting the code over the task), create a blackout_dates table (assuming it's needed), assume calendar year with no carryover. But **I'm making three assumptions, each of which could be wrong.**

### Friction Points Extracted

1. **ask** — Policy max: 25 or 30? Calendar or fiscal year? Carryover allowed?
2. **flag** — Task description contradicts codebase (30 vs 25)
3. **learned** — Chose to trust code over task; calendar year with no carryover

### Validation

- `ask` ✓ (multiple decisions needed)
- `flag` ✓ (contradiction found)
- `learned` ✓ (design assumptions recorded)

**Result:** 3/3 friction points map to 8-verb set. Continue looping.

---

## Simulation #23 (Ralph Loop Iteration 3)

**Project:** Podcast hosting platform (Django/Python)
**Feature:** listener-analytics
**Discipline:** data
**Task:** Build analytics dashboard showing listener demographics, episode performance (plays, downloads, completion rate), and subscriber churn prediction. Query data warehouse, cache results.
**Complication:** Race condition in data freshness

### Walkthrough

I start by reading the analytics requirements. I need to build endpoints that query listener data, aggregate by demographics, calculate completion rates. I find a `listeners` table and an `episode_plays` table. Good.

But I realize: **the task says "completion rate" but I don't see a `playback_progress` table or any way to know if someone finished an episode.** I search for how playback is tracked. I find a `player_sessions` table that has `started_at` and `ended_at` timestamps. **Does `ended_at` mean they finished listening, or just closed the app?** There's no way to know if someone actually listened to the full episode.

I also see the data warehouse comment. **Should I query the live database or a separate data warehouse?** The task says "query data warehouse, cache results" but there's no data warehouse connection in the codebase. Do I:
1. Create the DW connection
2. Assume it exists and mock it
3. Just query the live database

The task says DW, so I assume it exists. But **if it doesn't, my code will fail at runtime.**

I also realize: **if I cache results, how fresh is acceptable?** The task doesn't specify. Listener data changes in real-time. If I cache for 1 hour, the dashboard shows old data. If I cache for 1 minute, I'm hitting the warehouse constantly. **Where's the tradeoff documented?**

I implement it with a 15-minute cache (arbitrary choice) and add a "last updated" timestamp. I query the dashboard with `SELECT ... FROM listeners` assuming a DW exists. But **I'm making two assumptions: DW exists, and 15-minute freshness is acceptable.**

### Friction Points Extracted

1. **ask** — How is episode completion tracked? DW exists? What cache TTL?
2. **flag** — Playback completion metric undefined
3. **learned** — Chose 15-minute cache arbitrarily; needs validation
4. **blocked** — Data warehouse connection doesn't exist in codebase

### Validation

- `ask` ✓ (decisions needed)
- `flag` ✓ (undefined metric)
- `learned` ✓ (design assumption)
- `blocked` ✓ (external resource missing)

**Result:** 4/4 friction points map to 8-verb set. Continue looping.

---
