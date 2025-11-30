use std::borrow::Cow;
use std::sync::Arc;

use anyhow::anyhow;


pub struct OutputFileInfo<'t> {
    pub group_name: Arc<str>,
    pub path: Arc<str>,
    pub template: Arc<str>,
    pub params: Cow<'t, toml::Table>
}

fn get_output_file<'t>(group_name: Arc<str>, output_file: &'t toml::Table) -> anyhow::Result<OutputFileInfo<'t>> {
    let path = output_file.get("path")
        .and_then(|v| v.as_str())
        .map(|s| Arc::<str>::from(s))
        .ok_or_else(|| anyhow!("'path' property for output file is missing or is not a string"))?;

    let template = output_file.get("template")
        .and_then(|v| v.as_str())
        .map(|s| Arc::<str>::from(s))
        .ok_or_else(|| anyhow!("'template' property for output file is missing or is not a string"))?;

    let params = match output_file.get("params") {
        Some(v) => {v.as_table()
                .map(|t| Cow::Borrowed(t))
                .ok_or_else(|| anyhow!("'parms' property for output file is present, but is not a table"))?
            
        },
        None => Cow::Owned(toml::Table::new())
    };

    Ok(OutputFileInfo {
        group_name,
        path,
        template,
        params
    })
}

pub fn get_output_files(files_toml: &toml::Table) -> anyhow::Result<Vec<OutputFileInfo<'_>>> {
    let mut output_files: Vec<OutputFileInfo<'_>> = Vec::new();


    let files_table = files_toml.get("files")
        .and_then(|v| v.as_table())
        .ok_or_else(|| anyhow!("'files' property not found or is not a table"))?;

    for (file_group, val) in files_table {
        let file_group_arc = Arc::<str>::from(file_group.as_str());
        match val {
            toml::Value::Table(t) => {
                let output_file = get_output_file(file_group_arc.clone(), t)?;
                output_files.push(output_file);
            },
            toml::Value::Array(arr) => {
                for arr_val in arr {
                    let output_file_table = arr_val.as_table().ok_or_else(|| anyhow!("Output file array must be an array of table values"))?;
                    let output_file = get_output_file(file_group_arc.clone(), output_file_table)?;
                    output_files.push(output_file);
                }
            },
            _ => { return Err(anyhow!("Output file must be either a table or an array of table values")); }
        };
    }

    Ok(output_files)
}

