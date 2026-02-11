#!/usr/bin/env bun
/**
 * Ralph Task Signals MCP Server
 *
 * MCP server that forwards signal tool calls to the Ralph API server.
 * The API server handles SQLite writes and emits Tauri events for real-time frontend updates.
 *
 * Environment variables (required):
 *   RALPH_TASK_ID - Current task ID
 *   RALPH_SESSION_ID - Current session ID
 *   RALPH_API_PORT - Port of the Ralph API server
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js'
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js'
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js'

const taskId = process.env.RALPH_TASK_ID
const sessionId = process.env.RALPH_SESSION_ID
const apiPort = process.env.RALPH_API_PORT
const dbPath = process.env.RALPH_DB_PATH

if (!(taskId && sessionId && apiPort && dbPath)) {
  console.error('ERROR: Missing required environment variables')
  console.error('Required: RALPH_TASK_ID, RALPH_SESSION_ID, RALPH_API_PORT, RALPH_DB_PATH')
  process.exit(1)
}

const TASK_ID: string = taskId
const SESSION_ID: string = sessionId
const API_PORT: string = apiPort
const DB_PATH: string = dbPath

const API_URL = `http://127.0.0.1:${API_PORT}`

// Set the database path in the API server
await fetch(`${API_URL}/api/set-db-path`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ db_path: DB_PATH })
}).catch(err => {
  console.error('Failed to set database path in API server:', err)
  process.exit(1)
})

async function signal(verb: string, payload: Record<string, unknown>): Promise<string> {
  try {
    const response = await fetch(`${API_URL}/api/task-signal`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        task_id: Number.parseInt(TASK_ID, 10),
        session_id: SESSION_ID,
        verb,
        payload
      })
    })

    if (!response.ok) {
      const error = await response.json()
      throw new Error(error.error || `HTTP ${response.status}`)
    }

    return `Signal recorded: ${verb}`
  } catch (error) {
    console.error(`Failed to send ${verb} signal:`, error)
    throw error
  }
}

const server = new Server(
  {
    name: 'ralph-task-signals',
    version: '0.1.0'
  },
  {
    capabilities: {
      tools: {}
    }
  }
)

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: 'done',
      description: 'Signal that the task is fully complete and tested',
      inputSchema: {
        type: 'object',
        properties: {
          summary: {
            type: 'string',
            description: 'What was accomplished. Include key decisions and outcomes.'
          }
        },
        required: ['summary']
      }
    },
    {
      name: 'partial',
      description: 'Signal that progress was made but the task could not be fully completed',
      inputSchema: {
        type: 'object',
        properties: {
          summary: {
            type: 'string',
            description: 'What was accomplished so far.'
          },
          remaining: {
            type: 'string',
            description: 'What still needs to be done and why you stopped. Be specific.'
          }
        },
        required: ['summary', 'remaining']
      }
    },
    {
      name: 'stuck',
      description: 'Signal that you cannot make meaningful progress on this task',
      inputSchema: {
        type: 'object',
        properties: {
          reason: {
            type: 'string',
            description: "Why you're stuck. Be specific about what's blocking your ability to work."
          }
        },
        required: ['reason']
      }
    },
    {
      name: 'ask',
      description: 'Ask a question that you need answered to do this task well',
      inputSchema: {
        type: 'object',
        properties: {
          question: {
            type: 'string',
            description: 'The specific question you need answered.'
          },
          options: {
            type: 'array',
            items: { type: 'string' },
            description: "If you've identified possible answers, list them."
          },
          preferred: {
            type: 'string',
            description: 'If you have a recommendation, state it.'
          },
          blocking: {
            type: 'boolean',
            description:
              'True if you cannot proceed without the answer. False if nice-to-know but you can continue with your best judgment.'
          }
        },
        required: ['question', 'blocking']
      }
    },
    {
      name: 'flag',
      description: 'Report a problem you discovered during this task',
      inputSchema: {
        type: 'object',
        properties: {
          what: {
            type: 'string',
            description: 'Clear description of the problem.'
          },
          severity: {
            type: 'string',
            enum: ['info', 'warning', 'blocking'],
            description:
              "info (FYI, no action needed now), warning (should be addressed, doesn't block this task), blocking (this task can't be fully completed because of this)"
          },
          category: {
            type: 'string',
            enum: [
              'bug',
              'stale',
              'contradiction',
              'ambiguity',
              'overlap',
              'performance',
              'security',
              'incomplete_prior'
            ],
            description: 'Problem category for human reviewer.'
          }
        },
        required: ['what', 'severity', 'category']
      }
    },
    {
      name: 'learned',
      description: 'Record knowledge that will be useful for future tasks on this project',
      inputSchema: {
        type: 'object',
        properties: {
          text: {
            type: 'string',
            description:
              'The knowledge to record. Should be specific enough that a different developer on a different task would benefit from it.'
          },
          kind: {
            type: 'string',
            enum: ['discovery', 'decision', 'convention'],
            description:
              'discovery (factual finding about the codebase), decision (judgment call with rationale), convention (project pattern to follow)'
          },
          rationale: {
            type: 'string',
            description: 'For decisions: why you chose this approach and what alternatives you rejected.'
          },
          scope: {
            type: 'string',
            enum: ['project', 'feature', 'task'],
            description:
              "project (always relevant), feature (relevant to same feature's tasks), task (only relevant if this task is re-attempted). Default: feature."
          }
        },
        required: ['text', 'kind']
      }
    },
    {
      name: 'suggest',
      description: 'Recommend an action that should be taken but is outside the scope of your current task',
      inputSchema: {
        type: 'object',
        properties: {
          what: {
            type: 'string',
            description: 'What should be done.'
          },
          kind: {
            type: 'string',
            enum: ['new_task', 'split', 'refactor', 'alternative', 'deprecate'],
            description: 'Type of suggestion.'
          },
          why: {
            type: 'string',
            description: 'Why this is needed.'
          },
          feature: {
            type: 'string',
            description: 'Which feature this relates to (for new_task suggestions).'
          }
        },
        required: ['what', 'kind', 'why']
      }
    },
    {
      name: 'blocked',
      description:
        'Report that you cannot complete part of your task because something outside your control is missing or broken',
      inputSchema: {
        type: 'object',
        properties: {
          on: {
            type: 'string',
            description: 'What is blocking you.'
          },
          kind: {
            type: 'string',
            enum: ['upstream_task', 'external'],
            description:
              "upstream_task (dependency on another task that's incomplete) or external (credentials, services, human decisions, infrastructure)"
          },
          detail: {
            type: 'string',
            description: 'Additional context about the blocker.'
          }
        },
        required: ['on', 'kind']
      }
    }
  ]
}))

server.setRequestHandler(CallToolRequestSchema, async request => {
  const { name, arguments: args } = request.params

  try {
    const result = await signal(name, args as Record<string, unknown>)
    return {
      content: [{ type: 'text', text: result }]
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    return {
      content: [{ type: 'text', text: `Error: ${message}` }],
      isError: true
    }
  }
})

async function main() {
  const transport = new StdioServerTransport()
  await server.connect(transport)
  console.error('Ralph Task Signals MCP Server running')
}

main().catch(error => {
  console.error('Fatal error:', error)
  process.exit(1)
})
