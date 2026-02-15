use crate::types::McpServerConfig;
use crate::SqliteDb;
use ralph_errors::{codes, err_string, RalphError, RalphResult, RalphResultExt};
use rusqlite::OptionalExtension;

impl SqliteDb {
    pub(crate) fn check_exists(&self, table: &str, field: &str, value: &str) -> RalphResult<bool> {
        self.conn
            .query_row(
                &format!("SELECT COUNT(*) > 0 FROM {table} WHERE {field} = ?1"),
                [value],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check existence")
    }

    pub(crate) fn check_exists_excluding(
        &self,
        table: &str,
        field: &str,
        value: &str,
        exclude_field: &str,
        exclude_value: &str,
    ) -> RalphResult<bool> {
        self.conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) > 0 FROM {table} WHERE {field} = ?1 AND {exclude_field} != ?2"
                ),
                rusqlite::params![value, exclude_value],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check existence")
    }

    pub(crate) fn insert_string_list(
        &self,
        table: &str,
        fk_column: &str,
        fk_id: i64,
        value_column: &str,
        values: &[String],
    ) -> RalphResult<()> {
        for value in values {
            self.conn
                .execute(
                    &format!("INSERT INTO {table} ({fk_column}, {value_column}) VALUES (?1, ?2)"),
                    rusqlite::params![fk_id, value],
                )
                .ralph_err(codes::DB_WRITE, &format!("Failed to insert into {table}"))?;
        }
        Ok(())
    }

    pub(crate) fn read_string_list(
        &self,
        table: &str,
        fk_column: &str,
        fk_id: i64,
        value_column: &str,
    ) -> RalphResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {value_column} FROM {table} WHERE {fk_column} = ? ORDER BY id"
            ))
            .ralph_err(
                codes::DB_READ,
                &format!("Failed to prepare {table} list query"),
            )?;

        let rows = stmt
            .query_map([fk_id], |row| row.get::<_, String>(0))
            .ralph_err(
                codes::DB_READ,
                &format!("Failed to query {table} list rows"),
            )?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.ralph_err(
                codes::DB_READ,
                &format!("Failed to decode {table} list row"),
            )?);
        }
        Ok(out)
    }

    pub(crate) fn insert_mcp_servers(
        &self,
        discipline_id: i64,
        mcp_servers: &[McpServerConfig],
    ) -> RalphResult<()> {
        for mcp in mcp_servers {
            self.conn
                .execute(
                    "INSERT INTO discipline_mcp_servers (discipline_id, name, command) VALUES (?1, ?2, ?3)",
                    rusqlite::params![discipline_id, mcp.name, mcp.command],
                )
                .ralph_err(codes::DB_WRITE, "Failed to insert MCP server")?;

            let server_id = self.conn.last_insert_rowid();

            for (idx, arg) in mcp.args.iter().enumerate() {
                let order = i64::try_from(idx).map_err(|_| {
                    RalphError::new(
                        codes::INTERNAL,
                        format!("MCP args index out of range for sqlite i64: {idx}"),
                    )
                })?;
                self.conn
                    .execute(
                        "INSERT INTO discipline_mcp_server_args (server_id, arg, arg_order) VALUES (?1, ?2, ?3)",
                        rusqlite::params![server_id, arg, order],
                    )
                    .ralph_err(codes::DB_WRITE, "Failed to insert MCP arg")?;
            }

            for (key, value) in &mcp.env {
                self.conn
                    .execute(
                        "INSERT INTO discipline_mcp_server_env (server_id, key, value) VALUES (?1, ?2, ?3)",
                        rusqlite::params![server_id, key, value],
                    )
                    .ralph_err(codes::DB_WRITE, "Failed to insert MCP env")?;
            }
        }
        Ok(())
    }

    pub(crate) fn read_mcp_servers(&self, discipline_id: i64) -> RalphResult<Vec<McpServerConfig>> {
        let mut mcp_stmt = self.conn.prepare(
            "SELECT id, name, command FROM discipline_mcp_servers WHERE discipline_id = ? ORDER BY id",
        ).ralph_err(codes::DB_READ, "Failed to prepare MCP servers query")?;

        let mcp_rows = mcp_stmt
            .query_map([discipline_id], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .ralph_err(codes::DB_READ, "Failed to query MCP servers")?;

        let mut mcp_servers = vec![];
        for mcp_row in mcp_rows {
            let (server_id, name, command) =
                mcp_row.ralph_err(codes::DB_READ, "Failed to decode MCP server row")?;

            let mut arg_stmt = self.conn.prepare(
                "SELECT arg FROM discipline_mcp_server_args WHERE server_id = ? ORDER BY arg_order",
            ).ralph_err(codes::DB_READ, "Failed to prepare MCP args query")?;
            let arg_rows = arg_stmt
                .query_map([server_id], |r| r.get::<_, String>(0))
                .ralph_err(codes::DB_READ, "Failed to query MCP args")?;
            let mut args = Vec::new();
            for arg in arg_rows {
                args.push(arg.ralph_err(codes::DB_READ, "Failed to decode MCP arg")?);
            }

            let mut env_stmt = self
                .conn
                .prepare("SELECT key, value FROM discipline_mcp_server_env WHERE server_id = ?")
                .ralph_err(codes::DB_READ, "Failed to prepare MCP env query")?;
            let env_rows = env_stmt
                .query_map([server_id], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                })
                .ralph_err(codes::DB_READ, "Failed to query MCP env")?;
            let mut env = std::collections::HashMap::new();
            for row in env_rows {
                let (k, v) = row.ralph_err(codes::DB_READ, "Failed to decode MCP env row")?;
                env.insert(k, v);
            }

            mcp_servers.push(McpServerConfig {
                name,
                command,
                args,
                env,
            });
        }

        Ok(mcp_servers)
    }

    #[allow(dead_code)]
    pub(crate) fn get_id_from_name(&self, table: &str, name: &str) -> RalphResult<i64> {
        self.conn
            .query_row(
                &format!("SELECT id FROM {table} WHERE name = ?1"),
                [name],
                |row| row.get(0),
            )
            .optional()
            .ralph_err(codes::DB_READ, &format!("Failed to query {table}"))?
            .ok_or_else(|| err_string(codes::DB_READ, format!("{table} '{name}' does not exist")))
    }
}
