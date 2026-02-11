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
