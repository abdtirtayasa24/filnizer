use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::AppDatabase;
use crate::domain::files::FileCategory;
use crate::errors::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizerRule {
    pub id: String,
    pub kind: RuleKind,
    pub value: String,
    pub category: FileCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuleKind {
    Extension,
    FilenameContains,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveOrganizerRulesRequest {
    pub rules: Vec<OrganizerRuleInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizerRuleInput {
    pub id: Option<String>,
    pub kind: RuleKind,
    pub value: String,
    pub category: FileCategory,
}

pub struct OrganizerRulesRepository {
    database: AppDatabase,
}

impl OrganizerRulesRepository {
    pub fn new(database: AppDatabase) -> Self {
        Self { database }
    }

    pub fn list_rules(&self) -> Result<Vec<OrganizerRule>, AppError> {
        self.database.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT id, kind, value, category
                 FROM organizer_rules
                 ORDER BY created_at_unix_ms ASC",
            )?;
            let rows = statement.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?;

            let mut rules = Vec::new();
            for row in rows {
                let (id, kind, value, category) = row?;
                rules.push(OrganizerRule {
                    id,
                    kind: serde_json::from_str(&kind)
                        .map_err(|error| AppError::Unexpected(error.to_string()))?,
                    value,
                    category: serde_json::from_str(&category)
                        .map_err(|error| AppError::Unexpected(error.to_string()))?,
                });
            }
            Ok(rules)
        })
    }

    pub fn replace_rules(
        &self,
        rules: &[OrganizerRuleInput],
    ) -> Result<Vec<OrganizerRule>, AppError> {
        let normalized_rules = normalize_rules(rules)?;

        self.database.with_connection(|connection| {
            connection.execute("DELETE FROM organizer_rules", [])?;
            for rule in &normalized_rules {
                connection.execute(
                    "INSERT INTO organizer_rules (id, kind, value, category, created_at_unix_ms)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        rule.id,
                        serde_json::to_string(&rule.kind)
                            .map_err(|error| AppError::Unexpected(error.to_string()))?,
                        rule.value,
                        serde_json::to_string(&rule.category)
                            .map_err(|error| AppError::Unexpected(error.to_string()))?,
                        current_unix_ms(),
                    ],
                )?;
            }
            Ok(())
        })?;

        Ok(normalized_rules)
    }
}

fn normalize_rules(rules: &[OrganizerRuleInput]) -> Result<Vec<OrganizerRule>, AppError> {
    let mut normalized_rules = Vec::new();
    for rule in rules {
        let value = normalize_rule_value(rule.kind, &rule.value)?;
        normalized_rules.push(OrganizerRule {
            id: rule
                .id
                .clone()
                .unwrap_or_else(|| Uuid::now_v7().to_string()),
            kind: rule.kind,
            value,
            category: rule.category.clone(),
        });
    }
    Ok(normalized_rules)
}

fn normalize_rule_value(kind: RuleKind, value: &str) -> Result<String, AppError> {
    let value = value.trim().trim_start_matches('.').to_ascii_lowercase();
    if value.is_empty() {
        return Err(AppError::validation("Rule value cannot be empty"));
    }

    match kind {
        RuleKind::Extension | RuleKind::FilenameContains => Ok(value),
    }
}

fn current_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{OrganizerRuleInput, OrganizerRulesRepository, RuleKind};
    use crate::db::AppDatabase;
    use crate::domain::files::FileCategory;

    #[test]
    fn rules_round_trip_normalizes_extension() {
        let temp_dir = tempfile::tempdir().unwrap();
        let database = AppDatabase::open(temp_dir.path().join("test.sqlite3")).unwrap();
        let repository = OrganizerRulesRepository::new(database);

        let rules = repository
            .replace_rules(&[OrganizerRuleInput {
                id: None,
                kind: RuleKind::Extension,
                value: ".JPG".to_string(),
                category: FileCategory::Documents,
            }])
            .unwrap();

        assert_eq!(rules[0].value, "jpg");
        assert_eq!(repository.list_rules().unwrap(), rules);
    }
}
