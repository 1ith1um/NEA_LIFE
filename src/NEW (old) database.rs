use crate::grid::{Cell, CellType, Organism};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ── Data structures ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldState {
    pub organisms: Vec<Organism>,
    pub free_cells: Vec<Cell>,
    pub version: usize,
}

/// Lightweight metadata returned by `list_saves()` — no heavy cell data.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveInfo {
    pub id: usize,
    pub version: usize,
    pub name: Option<String>,
    pub timestamp: String,
    pub organism_count: usize,
    pub cell_count: usize,
}

// ── Validation ───────────────────────────────────────────────────────────────

impl Organism {
    pub fn is_valid(&self) -> bool {
        // Fixed: previously called self.id.unwrap() which would panic on None.
        !self.cells.is_empty()
    }
}

impl WorldState {
    /// Ensures no duplicate organism IDs and no overlapping cell positions.
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for org in &self.organisms {
            if !org.is_valid() {
                return Err(format!("Invalid organism: {:?}", org));
            }

            // Only check IDs if they exist
            if let Some(id) = org.id {
                if !ids.insert(id) {
                    return Err(format!("Duplicate organism ID: {}", id));
                }
            }
        }

        // Just collect positions — don't error
        let mut cell_positions = HashSet::new();

        for org in &self.organisms {
            for cell in &org.cells {
                cell_positions.insert((cell.i, cell.j));
            }
        }

        for cell in &self.free_cells {
            cell_positions.insert((cell.i, cell.j));
        }
        Ok(())
    }
}

impl std::fmt::Display for SaveInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(n) => write!(f, "{} — v{} ({})", n, self.version, &self.timestamp[..16]),
            None => write!(
                f,
                "v{} — {} organisms ({})",
                self.version,
                self.organism_count,
                &self.timestamp[..16]
            ),
        }
    }
}

// ── Database ─────────────────────────────────────────────────────────────────

pub struct WorldDatabase {
    conn: Connection,
}

impl WorldDatabase {
    /// Opens (or creates) the database at `path` and initialises all tables.
    ///
    /// Schema overview:
    ///  - `world_states`       — one row per save slot, with optional human-readable name
    ///  - `organisms`          — each organism belongs to a world_state via FK
    ///  - `cells`              — organism cells link via organism_id;
    ///                           free cells carry a world_state_id directly
    ///  - `population_history` — time-series data for the population graph
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             BEGIN;

             CREATE TABLE IF NOT EXISTS world_states (
                 id         INTEGER PRIMARY KEY AUTOINCREMENT,
                 version    INTEGER NOT NULL,
                 name       TEXT,
                 timestamp  DATETIME DEFAULT CURRENT_TIMESTAMP
             );

             CREATE TABLE IF NOT EXISTS organisms (
                 id             INTEGER PRIMARY KEY AUTOINCREMENT,
                 world_state_id INTEGER NOT NULL,
                 energy         INTEGER NOT NULL,
                 able_to_move   BOOLEAN NOT NULL,
                 FOREIGN KEY (world_state_id) REFERENCES world_states(id) ON DELETE CASCADE
             );

             -- Organism cells: linked via organism_id (world_state_id is NULL).
             -- Free cells:     organism_id is NULL; world_state_id identifies the save.
             CREATE TABLE IF NOT EXISTS cells (
                 id             INTEGER PRIMARY KEY AUTOINCREMENT,
                 i              INTEGER NOT NULL,
                 j              INTEGER NOT NULL,
                 cell_type      TEXT    NOT NULL,
                 organism_id    INTEGER,
                 world_state_id INTEGER,
                 FOREIGN KEY (organism_id)    REFERENCES organisms(id)    ON DELETE CASCADE,
                 FOREIGN KEY (world_state_id) REFERENCES world_states(id) ON DELETE CASCADE
             );

             CREATE TABLE IF NOT EXISTS population_history (
                 id         INTEGER PRIMARY KEY AUTOINCREMENT,
                 tick       INTEGER  NOT NULL,
                 population INTEGER  NOT NULL,
                 timestamp  DATETIME DEFAULT CURRENT_TIMESTAMP
             );

             COMMIT;",
        )?;

        Ok(Self { conn })
    }

    // ── Saving ───────────────────────────────────────────────────────────────

    /// Saves a new snapshot of the world. Returns the new save's database ID.
    ///
    /// Each call appends a fresh row to `world_states` rather than replacing
    /// the previous one, so the full save history is preserved.  Pass an
    /// optional `name` (e.g. `"before big experiment"`) to label the slot.
    pub fn save_state(&mut self, state: &WorldState, name: Option<&str>) -> Result<usize> {
        let tx = self.conn.transaction()?;

        // Insert a new world_states row.
        tx.execute(
            "INSERT INTO world_states (version, name) VALUES (?1, ?2)",
            params![state.version, name],
        )?;
        let world_state_id = tx.last_insert_rowid() as usize;

        // Insert organisms and record the DB-assigned IDs.
        let mut organism_db_ids = Vec::with_capacity(state.organisms.len());
        for organism in &state.organisms {
            tx.execute(
                "INSERT INTO organisms (world_state_id, energy, able_to_move)
                 VALUES (?1, ?2, ?3)",
                params![world_state_id, organism.energy, organism.able_to_move],
            )?;
            organism_db_ids.push(tx.last_insert_rowid() as usize);
        }

        // Collect the newest cell at each (i, j) — organism cells first, then
        // free cells, so free cells win on any overlap (matching prior behaviour).
        let mut newest: HashMap<(isize, isize), (Cell, Option<usize>, Option<usize>)> =
            HashMap::new();

        for (org_idx, organism) in state.organisms.iter().enumerate() {
            let db_id = organism_db_ids[org_idx];
            for cell in &organism.cells {
                newest.insert((cell.i, cell.j), (cell.clone(), Some(db_id), None));
            }
        }
        for cell in &state.free_cells {
            newest.insert((cell.i, cell.j), (cell.clone(), None, Some(world_state_id)));
        }

        for ((i, j), (cell, organism_id, ws_id)) in newest {
            tx.execute(
                "INSERT INTO cells (i, j, cell_type, organism_id, world_state_id)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    i,
                    j,
                    serde_json::to_string(&cell.cell_type)
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                    organism_id,
                    ws_id,
                ],
            )?;
        }

        tx.commit()?;
        Ok(world_state_id)
    }

    // ── Loading ──────────────────────────────────────────────────────────────

    /// Loads the most recently saved world state.
    pub fn load_latest_state(&self) -> Result<WorldState> {
        let id: usize = self.conn.query_row(
            "SELECT id FROM world_states ORDER BY timestamp DESC LIMIT 1",
            [],
            |row| row.get(0),
        )?;
        self.load_state_by_id(id)
    }

    /// Loads a world state by its specific save-slot ID (as returned by
    /// `save_state` or listed by `list_saves`).
    pub fn load_state_by_id(&self, world_state_id: usize) -> Result<WorldState> {
        let version: usize = self.conn.query_row(
            "SELECT version FROM world_states WHERE id = ?1",
            params![world_state_id],
            |row| row.get(0),
        )?;

        // Load organism cells.
        let mut stmt = self.conn.prepare(
            "SELECT c.i, c.j, c.cell_type, c.organism_id
             FROM cells c
             JOIN organisms o ON c.organism_id = o.id
             WHERE o.world_state_id = ?1 AND c.organism_id IS NOT NULL",
        )?;

        let mut organism_cells: HashMap<usize, Vec<Cell>> = HashMap::new();
        for row in stmt.query_map(params![world_state_id], |row| {
            let cell_type_str: String = row.get(2)?;
            let cell_type = serde_json::from_str(&cell_type_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok((
                Cell {
                    i: row.get(0)?,
                    j: row.get(1)?,
                    cell_type,
                },
                row.get::<_, usize>(3)?,
            ))
        })? {
            let (cell, org_id) = row?;
            organism_cells.entry(org_id).or_default().push(cell);
        }

        // Load organisms.
        let mut stmt = self
            .conn
            .prepare("SELECT id, energy, able_to_move FROM organisms WHERE world_state_id = ?1")?;
        let organisms = stmt
            .query_map(params![world_state_id], |row| {
                let id: usize = row.get(0)?;
                let cells = organism_cells.remove(&id).unwrap_or_default();
                Ok(Organism {
                    id: Some(id),
                    cells,
                    energy: row.get(1)?,
                    able_to_move: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        // Load free cells (organism_id IS NULL, world_state_id matches).
        let mut stmt = self.conn.prepare(
            "SELECT i, j, cell_type FROM cells
             WHERE world_state_id = ?1 AND organism_id IS NULL",
        )?;
        let free_cells = stmt
            .query_map(params![world_state_id], |row| {
                let cell_type_str: String = row.get(2)?;
                let cell_type = serde_json::from_str(&cell_type_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
                Ok(Cell {
                    i: row.get(0)?,
                    j: row.get(1)?,
                    cell_type,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(WorldState {
            organisms,
            free_cells,
            version,
        })
    }

    // ── Save management ──────────────────────────────────────────────────────

    /// Returns lightweight metadata for every save slot, newest first.
    pub fn list_saves(&self) -> Result<Vec<SaveInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                 ws.id,
                 ws.version,
                 ws.name,
                 ws.timestamp,
                 COUNT(DISTINCT o.id)                                   AS organism_count,
                 COUNT(c.id)                                            AS cell_count
             FROM world_states ws
             LEFT JOIN organisms o ON o.world_state_id = ws.id
             LEFT JOIN cells c     ON c.organism_id = o.id
                                   OR c.world_state_id = ws.id
             GROUP BY ws.id
             ORDER BY ws.timestamp DESC",
        )?;

        let saves = stmt
            .query_map([], |row| {
                Ok(SaveInfo {
                    id: row.get(0)?,
                    version: row.get(1)?,
                    name: row.get(2)?,
                    timestamp: row.get(3)?,
                    organism_count: row.get(4)?,
                    cell_count: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(saves)
    }

    /// Deletes a specific save slot and all its associated data (cascades).
    pub fn delete_state(&mut self, world_state_id: usize) -> Result<()> {
        self.conn.execute(
            "DELETE FROM world_states WHERE id = ?1",
            params![world_state_id],
        )?;
        Ok(())
    }

    /// Keeps only the `keep_n` most recent saves, deleting the rest.
    /// Useful for preventing unbounded database growth during long simulations.
    pub fn prune_old_saves(&mut self, keep_n: usize) -> Result<usize> {
        let deleted = self.conn.execute(
            "DELETE FROM world_states
             WHERE id NOT IN (
                 SELECT id FROM world_states
                 ORDER BY timestamp DESC
                 LIMIT ?1
             )",
            params![keep_n],
        )?;
        Ok(deleted)
    }

    // ── Population history ───────────────────────────────────────────────────

    /// Records a population count at a given simulation tick.
    /// Call this from `LifeSim::update` alongside `population_graph.update()`.
    pub fn record_population(&mut self, tick: usize, population: usize) -> Result<()> {
        self.conn.execute(
            "INSERT INTO population_history (tick, population) VALUES (?1, ?2)",
            params![tick, population],
        )?;
        Ok(())
    }

    /// Returns the full population history as `(tick, population)` pairs,
    /// ordered from earliest to latest. Feed these into `PopulationGraph` on
    /// `LoadState` to restore the graph alongside the world.
    pub fn get_population_history(&self) -> Result<Vec<(usize, usize)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tick, population FROM population_history ORDER BY tick ASC")?;

        let history = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>>>()?;

        Ok(history)
    }

    /// Clears all population history (e.g. when the simulation is reset).
    pub fn clear_population_history(&mut self) -> Result<()> {
        self.conn.execute("DELETE FROM population_history", [])?;
        Ok(())
    }
}
