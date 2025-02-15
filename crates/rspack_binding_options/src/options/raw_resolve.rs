use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{Alias, AliasMap, Resolve};
use serde::Deserialize;

pub type AliasValue = serde_json::Value;

type RawAliasOption = HashMap<String, Vec<AliasValue>>;
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawResolveOptions {
  pub prefer_relative: Option<bool>,
  pub extensions: Option<Vec<String>>,
  pub main_files: Option<Vec<String>>,
  pub main_fields: Option<Vec<String>>,
  pub browser_field: Option<bool>,
  pub condition_names: Option<Vec<String>>,
  #[serde(serialize_with = "ordered_map")]
  #[napi(ts_type = "Record<string, Array<string | false>>")]
  pub alias: Option<RawAliasOption>,
  #[serde(serialize_with = "ordered_map")]
  #[napi(ts_type = "Record<string, Array<string | false>>")]
  pub fallback: Option<RawAliasOption>,
  pub symlinks: Option<bool>,
  pub ts_config_path: Option<String>,
  pub modules: Option<Vec<String>>,
}

fn normalize_alias(alias: Option<RawAliasOption>) -> anyhow::Result<Option<Alias>> {
  alias
    .map(|alias| {
      alias
        .into_iter()
        .map(|(key, array)| {
          array
            .into_iter()
            .map(|value| {
              if let Some(s) = value.as_str() {
                Ok(AliasMap::Target(s.to_string()))
              } else if let Some(b) = value.as_bool() {
                if b {
                  Err(anyhow::Error::msg(format!(
                    "Alias should not be true in {key}"
                  )))
                } else {
                  Ok(AliasMap::Ignored)
                }
              } else {
                Err(anyhow::Error::msg(format!(
                  "Alias should be false or string in {key}"
                )))
              }
            })
            .collect::<anyhow::Result<_>>()
            .map(|value| (key, value))
        })
        .collect::<anyhow::Result<_>>()
    })
    .map_or(Ok(None), |v| v.map(Some))
}

impl TryFrom<RawResolveOptions> for Resolve {
  type Error = rspack_error::Error;

  fn try_from(value: RawResolveOptions) -> Result<Self, Self::Error> {
    let prefer_relative = value.prefer_relative;
    let extensions = value.extensions;
    let browser_field = value.browser_field;
    let main_files = value.main_files;
    let main_fields = value.main_fields;
    let condition_names = value.condition_names;
    let symlinks = value.symlinks;
    let alias = normalize_alias(value.alias)?;
    let fallback = normalize_alias(value.fallback)?;
    let modules = value.modules;
    let tsconfig = value.ts_config_path.map(std::path::PathBuf::from);
    Ok(Resolve {
      modules,
      prefer_relative,
      extensions,
      browser_field,
      main_fields,
      main_files,
      condition_names,
      alias,
      symlinks,
      tsconfig,
      fallback,
    })
  }
}
