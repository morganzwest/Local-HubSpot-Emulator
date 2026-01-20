use anyhow::Result;

use crate::inline::{InlineConfig, InlineLanguage};

pub fn validate_inline_config(cfg: &InlineConfig) -> Result<()> {
    // ---------- action ----------
    if cfg.action.source.trim().is_empty() {
        anyhow::bail!("action.source must not be empty");
    }

    let ext = cfg
        .action
        .entry
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match cfg.action.language {
        InlineLanguage::Js => {
            if !matches!(ext.as_str(), "js" | "mjs" | "cjs") {
                anyhow::bail!(
                    "JavaScript action must use .js/.mjs/.cjs (got '{}')",
                    cfg.action.entry
                );
            }
        }
        InlineLanguage::Python => {
            if ext != "py" {
                anyhow::bail!(
                    "Python action must use .py (got '{}')",
                    cfg.action.entry
                );
            }
        }
    }

    // ---------- fixtures ----------
    if cfg.fixtures.is_empty() {
        anyhow::bail!("At least one fixture must be provided");
    }

    for fixture in &cfg.fixtures {
        if fixture.name.trim().is_empty() {
            anyhow::bail!("Fixture name must not be empty");
        }

        if fixture.source.trim().is_empty() {
            anyhow::bail!("Fixture '{}' is empty", fixture.name);
        }

        serde_json::from_str::<serde_json::Value>(&fixture.source)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Fixture '{}' contains invalid JSON: {}",
                    fixture.name,
                    e
                )
            })?;
    }

    // ---------- repeat ----------
    if cfg.repeat == 0 {
        anyhow::bail!("repeat must be >= 1");
    }

    Ok(())
}
