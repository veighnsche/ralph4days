# MCP Servers Configuration

This project uses Model Context Protocol (MCP) servers to enhance AI-assisted development.

## Configured Servers

### 1. **shadcn/ui MCP Server** ✅
- **Package:** `@jpisnice/shadcn-ui-mcp-server`
- **Status:** Connected
- **Purpose:** Direct access to latest shadcn/ui component implementations
- **Benefits:**
  - On-demand component retrieval with latest code
  - Demo code and usage examples
  - Framework-specific installation guides (Next.js, Vite, Remix)
  - Package manager support (npm, yarn, pnpm, bun)
- **Documentation:** https://ui.shadcn.com/docs/mcp
- **Source:** https://github.com/Jpisnice/shadcn-ui-mcp-server

### 2. **Tailwind CSS MCP Server** ✅
- **Package:** `tailwindcss-mcp-server`
- **Status:** Connected
- **Purpose:** Tailwind CSS utilities and documentation
- **Benefits:**
  - Tailwind CSS v4 support
  - Utility classes and documentation
  - Template generation capabilities
- **Documentation:** https://www.npmjs.com/package/tailwindcss-mcp-server

### 3. **Sequential Thinking MCP** ✅
- **Package:** `@modelcontextprotocol/server-sequential-thinking`
- **Status:** Connected
- **Purpose:** Structured problem-solving and architectural decisions
- **Benefits:**
  - Reflective thinking process for complex problems
  - Help with architectural decisions
  - Better debugging of elusive issues
  - Planning large-scale features
- **Documentation:** https://apidog.com/blog/top-10-mcp-servers-for-claude-code/

## Configuration

MCP servers are configured in `.mcp.json` at the project root. This file is checked into git so all team members benefit from the same tooling.

### File Location
```
ralph4days/
└── .mcp.json  # Project-scoped MCP configuration
```

### Scope
- **Project-scoped**: Shared with everyone working on this project
- These servers are automatically available when you open this project in Claude Code

## Usage

MCP servers work transparently - Claude Code automatically uses them when relevant. For example:

- Ask about shadcn/ui components → shadcn-ui MCP provides latest code
- Need React 19 documentation → Context7 fetches current docs
- Working with Tailwind → tailwindcss MCP provides utilities
- Making architectural decisions → Sequential Thinking structures the analysis

## Verification

Check server status with:
```bash
claude mcp list
```

## Additional Resources

- [Claude Code MCP Documentation](https://code.claude.com/docs/en/mcp)
- [MCP Server Finder](https://www.mcpserverfinder.com/)
- [Awesome MCP Servers](https://mcp-awesome.com/)
- [The New Stack - 10 MCP Servers for Frontend Developers](https://thenewstack.io/10-mcp-servers-for-frontend-developers/)

## Notes

- MCP servers run on-demand and don't affect startup time
- They use npx with `-y` flag for automatic package installation
- No manual npm install needed - packages are fetched as needed
