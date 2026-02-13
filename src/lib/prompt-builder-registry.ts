export type SectionCategory = 'project' | 'subsystem' | 'task' | 'discipline' | 'state' | 'user' | 'instructions'

export interface SectionMeta {
  name: string
  displayName: string
  description: string
  category: SectionCategory
  isInstruction: boolean
}

export interface PromptBuilderDefinition {
  name: string
  label: string
  sections: string[]
}

export interface SectionSettings {
  enabled: boolean
  instructionOverride?: string | null
}

export interface PromptBuilderConfig {
  basePrompt: string
  sectionOrder: string[]
  sections: Record<string, SectionSettings>
}

export const SECTION_REGISTRY: SectionMeta[] = [
  {
    name: 'project_context',
    displayName: 'Project Context',
    description: 'CLAUDE.RALPH.md and project-level context',
    category: 'project',
    isInstruction: false
  },
  {
    name: 'project_metadata',
    displayName: 'Project Metadata',
    description: 'Project title, description, and creation date',
    category: 'project',
    isInstruction: false
  },
  {
    name: 'codebase_state',
    displayName: 'Codebase Snapshot',
    description: 'Filesystem tree and tech stack analysis',
    category: 'project',
    isInstruction: false
  },
  {
    name: 'feature_listing',
    displayName: 'Subsystem Listing',
    description: 'All subsystems with descriptions and stats',
    category: 'subsystem',
    isInstruction: false
  },
  {
    name: 'feature_context',
    displayName: 'Subsystem Context',
    description: 'Target subsystem details, architecture, and learnings',
    category: 'subsystem',
    isInstruction: false
  },
  {
    name: 'feature_files',
    displayName: 'Subsystem Files',
    description: 'Inlined contents of subsystem context files',
    category: 'subsystem',
    isInstruction: false
  },
  {
    name: 'feature_state',
    displayName: 'Subsystem State',
    description: 'Tasks grouped by status for the target subsystem',
    category: 'subsystem',
    isInstruction: false
  },
  {
    name: 'task_listing',
    displayName: 'Task Listing',
    description: 'All tasks with status, priority, and dependencies',
    category: 'task',
    isInstruction: false
  },
  {
    name: 'task_details',
    displayName: 'Task Details',
    description: 'Full details of the target task',
    category: 'task',
    isInstruction: false
  },
  {
    name: 'task_files',
    displayName: 'Task Files',
    description: 'Inlined contents of task context files',
    category: 'task',
    isInstruction: false
  },
  {
    name: 'dependency_context',
    displayName: 'Dependency Context',
    description: 'Details of tasks this task depends on',
    category: 'task',
    isInstruction: false
  },
  {
    name: 'previous_attempts',
    displayName: 'Previous Attempts',
    description: 'Comments from prior iterations on this task',
    category: 'task',
    isInstruction: false
  },
  {
    name: 'discipline_listing',
    displayName: 'Discipline Listing',
    description: 'All disciplines with skills and conventions',
    category: 'discipline',
    isInstruction: false
  },
  {
    name: 'discipline_persona',
    displayName: 'Discipline Persona',
    description: 'System prompt and identity for the target discipline',
    category: 'discipline',
    isInstruction: false
  },
  {
    name: 'state_files',
    displayName: 'State Files',
    description: 'Contents of progress.txt and learnings.txt',
    category: 'state',
    isInstruction: false
  },
  {
    name: 'user_input',
    displayName: 'User Input',
    description: 'Raw text from the user (braindump, yap, etc.)',
    category: 'user',
    isInstruction: false
  },
  {
    name: 'braindump_instructions',
    displayName: 'Braindump Instructions',
    description: 'Instructions for structuring a raw braindump into tasks',
    category: 'instructions',
    isInstruction: true
  },
  {
    name: 'yap_instructions',
    displayName: 'Yap Instructions',
    description: 'Instructions for creating/updating tasks from user input',
    category: 'instructions',
    isInstruction: true
  },
  {
    name: 'ramble_instructions',
    displayName: 'Ramble Instructions',
    description: 'Instructions for creating/updating subsystems from user input',
    category: 'instructions',
    isInstruction: true
  },
  {
    name: 'discuss_instructions',
    displayName: 'Discuss Instructions',
    description: 'Instructions for updating discipline configurations',
    category: 'instructions',
    isInstruction: true
  },
  {
    name: 'task_exec_instructions',
    displayName: 'Task Exec Instructions',
    description: 'Instructions for executing a specific task',
    category: 'instructions',
    isInstruction: true
  },
  {
    name: 'opus_review_instructions',
    displayName: 'Opus Review Instructions',
    description: 'Instructions for reviewing recent work quality',
    category: 'instructions',
    isInstruction: true
  }
]

export const PROMPT_BUILDER_REGISTRY: PromptBuilderDefinition[] = [
  {
    name: 'braindump',
    label: 'Braindump',
    sections: [
      'project_context',
      'project_metadata',
      'codebase_state',
      'feature_listing',
      'discipline_listing',
      'user_input',
      'braindump_instructions'
    ]
  },
  {
    name: 'yap',
    label: 'Yap',
    sections: [
      'project_context',
      'project_metadata',
      'feature_listing',
      'task_listing',
      'discipline_listing',
      'user_input',
      'yap_instructions'
    ]
  },
  {
    name: 'ramble',
    label: 'Ramble',
    sections: [
      'project_context',
      'project_metadata',
      'feature_listing',
      'feature_state',
      'user_input',
      'ramble_instructions'
    ]
  },
  {
    name: 'discuss',
    label: 'Discuss',
    sections: ['project_context', 'project_metadata', 'discipline_listing', 'user_input', 'discuss_instructions']
  },
  {
    name: 'task_execution',
    label: 'Task Execution',
    sections: [
      'project_context',
      'discipline_persona',
      'feature_context',
      'feature_files',
      'feature_state',
      'state_files',
      'previous_attempts',
      'dependency_context',
      'task_details',
      'task_files',
      'task_exec_instructions'
    ]
  },
  {
    name: 'opus_review',
    label: 'Opus Review',
    sections: [
      'project_context',
      'feature_context',
      'feature_files',
      'feature_state',
      'task_listing',
      'state_files',
      'opus_review_instructions'
    ]
  }
]

export const DEFAULT_INSTRUCTIONS: Record<string, string> = {
  braindump_instructions: `## Instructions

You are receiving a raw braindump from the user. Your job is to analyze it and create structured project data.

### What to do

1. **Read the braindump carefully.** Identify distinct subsystems, areas of work, and concrete tasks.
2. **Create subsystems** using the available subsystem-management MCP tools in this session. Group related work into cohesive subsystems. Each subsystem should have a clear name, display name, and description.
3. **Create or update disciplines** using the available discipline-management MCP tools if the work requires disciplines beyond the defaults. Configure system_prompt, skills, and conventions for each.
4. **Create tasks** using the available task-management MCP tools. Each task should:
   - Belong to exactly one subsystem and one discipline
   - Have a clear, actionable title
   - Include a description explaining what needs to be done
   - List specific acceptance criteria
   - Set appropriate priority (low, medium, high, critical)
   - Specify dependencies on other tasks via \`depends_on\` where ordering matters
5. **Ask clarifying questions** if the braindump is ambiguous or incomplete. It is better to ask than to guess wrong.

### Guidelines

- Prefer many small, focused tasks over few large ones
- Each task should be completable in a single Claude session (1-10 turns)
- Set \`estimated_turns\` to help with scheduling
- Use \`context_files\` to point tasks at the relevant source files
- Use \`hints\` to give the executing agent useful tips
- Create dependencies between tasks when one must complete before another can start`,

  yap_instructions: `## Instructions

You are receiving additional input from the user about tasks. Review the existing tasks and the user's input, then create new tasks or update existing ones.

### What to do

1. **Review existing tasks** listed above to understand current project state.
2. **Interpret the user's input** in the context of existing subsystems and tasks.
3. **Create new tasks** using the available task-management MCP tools where the user describes new work.
4. **Update existing tasks** using the available task-management MCP tools where the user wants changes to current tasks (status, description, priority, acceptance criteria, etc.).
5. **Maintain consistency** with the existing subsystem and discipline structure.

### Guidelines

- Be specific about acceptance criteria -- vague criteria lead to vague implementations
- Set dependencies (\`depends_on\`) when tasks have ordering requirements
- Use \`context_files\` to point tasks at relevant source files
- If the user's input conflicts with existing tasks, ask for clarification
- Preserve existing task data when updating -- only change what the user explicitly requests
- Use \`hints\` to pass along any useful implementation tips from the user`,

  ramble_instructions: `## Instructions

You are receiving input from the user about subsystems. Review the existing subsystems and the user's input, then create or update subsystems as needed.

### What to do

1. **Review existing subsystems** listed above to understand current project structure.
2. **Interpret the user's input** about new or changed subsystems.
3. **Create new subsystems** using the available subsystem-management MCP tools where the user describes new areas of work.
4. **Update existing subsystems** using the available subsystem-management MCP tools where the user wants changes.
5. **Consider dependencies** between subsystems and how tasks should be organized.

### Guidelines

- Each subsystem should represent a cohesive area of work
- Use clear, descriptive names that convey the subsystem's purpose
- Set \`knowledge_paths\` to point at reference documents (specs, designs, docs)
- Set \`context_files\` to point at the key source files for the subsystem
- If a subsystem is being split or merged, update associated tasks accordingly
- Keep subsystem descriptions concise but informative`,

  discuss_instructions: `## Instructions

You are receiving input from the user about disciplines. Review the existing disciplines and the user's input, then update discipline configurations as needed.

### What to do

1. **Review existing disciplines** listed above to understand current configuration.
2. **Interpret the user's input** about discipline changes.
3. **Update disciplines** using the \`update_discipline\` MCP tool to modify configurations.
4. **Create new disciplines** using the \`create_discipline\` MCP tool if the user describes new roles.

### Focus areas

- **system_prompt**: The persona and instructions for agents working in this discipline. Should define the agent's role, expertise, and approach.
- **skills**: A list of specific capabilities the discipline brings (e.g., "TypeScript", "API design", "performance optimization").
- **conventions**: Coding standards, patterns, and practices the discipline enforces (e.g., "use early returns", "prefer composition over inheritance").
- **mcp_servers**: Additional MCP servers the discipline needs for specialized tooling.

### Guidelines

- System prompts should be detailed enough to guide a Claude agent effectively
- Skills should be specific and actionable, not vague
- Conventions should be concrete rules, not aspirational statements
- Keep discipline scope focused -- one discipline should not cover everything`,

  task_exec_instructions: `## Instructions

You are executing a specific task. Complete it thoroughly, following the discipline conventions and acceptance criteria.

### What to do

1. **Read the task details** above carefully. Understand the title, description, acceptance criteria, and hints.
2. **Follow the discipline conventions** specified in the "You Are" section.
3. **Implement the work** described in the task. Use context files and reference documents as guides.
4. **Verify acceptance criteria** are met before marking the task complete.
5. **Update task status** to \`done\` using the \`update_task\` MCP tool when complete.
6. **Commit your changes** with a descriptive commit message summarizing what was done.
7. **Append a summary** to \`progress.txt\` describing what you accomplished in this iteration.

### Rules

- Work on **ONE task only** per iteration. Do not start other tasks.
- If you encounter a blocker, update the task status to \`blocked\` with a \`blocked_by\` explanation and stop.
- If ALL tasks in the project are now complete, output \`<promise>COMPLETE</promise>\` at the end of your response.
- Do not modify files outside the scope of your assigned task unless absolutely necessary.
- If a dependency task is not yet complete, do not proceed -- mark yourself as blocked.`,

  opus_review_instructions: `## Instructions

You are reviewing recent work for quality. Focus on correctness, code quality, and completeness.

### What to do

1. **Review recently completed tasks.** Check that the work actually implements what the task describes.
2. **Verify the code works.** Run tests, check for compilation errors, and look for obvious bugs.
3. **Check code quality.** Look for:
   - Code that does not follow project conventions
   - Missing error handling
   - Hardcoded values that should be configurable
   - Dead code or unused imports
   - Poor naming or unclear logic
4. **Fix issues you find.** Make corrections directly rather than just noting them.
5. **Update learnings.txt.** Add patterns, gotchas, or useful discoveries to \`learnings.txt\` so future iterations benefit.
6. **Update task statuses** if you discover a "done" task that is not actually complete -- set it back to \`in_progress\` or \`blocked\` as appropriate.

### Guidelines

- Quality over speed. It is better to fix one thing well than to skim many things.
- Be specific in learnings -- "the X pattern causes Y problem, use Z instead" is useful; "be careful with X" is not.
- If you find systemic issues (e.g., a pattern repeated across many files), note it in learnings and fix the instances you find.
- Do not re-do completed work that is correct. Focus on finding and fixing actual problems.
- Commit fixes with clear messages explaining what was wrong and how it was fixed.`
}

export const CATEGORY_COLORS: Record<string, string> = {
  project: 'bg-blue-500/15 text-blue-700 dark:text-blue-400',
  subsystem: 'bg-violet-500/15 text-violet-700 dark:text-violet-400',
  task: 'bg-amber-500/15 text-amber-700 dark:text-amber-400',
  discipline: 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-400',
  state: 'bg-slate-500/15 text-slate-700 dark:text-slate-400',
  user: 'bg-rose-500/15 text-rose-700 dark:text-rose-400',
  instructions: 'bg-orange-500/15 text-orange-700 dark:text-orange-400'
}

export const CATEGORY_GRADIENT_COLORS: Record<string, string> = {
  project: 'rgba(59, 130, 246, 0.12)',
  subsystem: 'rgba(139, 92, 246, 0.12)',
  task: 'rgba(245, 158, 11, 0.12)',
  discipline: 'rgba(16, 185, 129, 0.12)',
  state: 'rgba(100, 116, 139, 0.12)',
  user: 'rgba(244, 63, 94, 0.12)',
  instructions: 'rgba(249, 115, 22, 0.12)'
}

export const BUILT_IN_PROMPT_BUILDERS = [
  { value: 'braindump', label: 'Braindump' },
  { value: 'yap', label: 'Yap' },
  { value: 'ramble', label: 'Ramble' },
  { value: 'discuss', label: 'Discuss' },
  { value: 'task_execution', label: 'Task Execution' },
  { value: 'opus_review', label: 'Opus Review' }
] as const

export function getDefaultPromptBuilderConfig(promptName: string): PromptBuilderConfig {
  const promptBuilder = PROMPT_BUILDER_REGISTRY.find(r => r.name === promptName)
  if (!promptBuilder) {
    return {
      basePrompt: promptName,
      sectionOrder: SECTION_REGISTRY.map(s => s.name),
      sections: Object.fromEntries(SECTION_REGISTRY.map(s => [s.name, { enabled: false }]))
    }
  }

  const enabledSet = new Set(promptBuilder.sections)
  const disabledSections = SECTION_REGISTRY.filter(s => !enabledSet.has(s.name)).map(s => s.name)
  const sectionOrder = [...promptBuilder.sections, ...disabledSections]

  return {
    basePrompt: promptName,
    sectionOrder,
    sections: Object.fromEntries(sectionOrder.map(name => [name, { enabled: enabledSet.has(name) }]))
  }
}
