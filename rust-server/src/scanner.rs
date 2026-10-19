use std::collections::BTreeMap;

use regex::Regex;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub text: String,
    pub name: String,
    pub default: Value,
    pub parameters: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub name: String,
    pub features: BTreeMap<String, Feature>,
    pub string: String,
    pub settings: DeviceSettings,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSettings {
    pub batch_mode: Setting<String>,
    pub filters: Setting<String>,
    pub pipeline: Setting<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Setting<T> {
    pub options: Vec<T>,
    pub default: Value,
}

pub fn parse_device_ids(output: &str) -> Vec<String> {
    let pattern = Regex::new(r"device `?(.*)' is a .*").expect("constant regex");
    output
        .lines()
        .filter_map(|line| {
            pattern
                .captures(line)
                .map(|captures| captures[1].to_owned())
        })
        .collect()
}

pub fn parse_device(id: &str, output: &str, pipeline_options: &[String]) -> Result<Device, String> {
    let line_pattern = Regex::new(r"^\s*([-]{1,2}[-a-zA-Z0-9]+) ?(.*?) \[(.*?)\](?: \[(.*?)\])?$")
        .expect("constant regex");
    let mut features = BTreeMap::new();

    for line in output.lines() {
        let Some(captures) = line_pattern.captures(line) else {
            continue;
        };
        let name = captures[1].to_owned();
        let parameters = captures[2].trim().to_owned();
        let default_text = captures[3].trim();
        let meta = captures.get(4).map(|value| value.as_str().trim());
        let enabled = default_text != "inactive" && meta != Some("read-only");
        let mut feature = Feature {
            text: line.trim().to_owned(),
            name: name.clone(),
            default: number_or_string(default_text),
            parameters: parameters
                .strip_prefix("auto|")
                .unwrap_or(&parameters)
                .to_owned(),
            enabled,
            options: None,
            limits: None,
            interval: None,
        };

        if !enabled {
            continue;
        }

        match name.as_str() {
            "--adf-mode" | "--mode" | "--source" => {
                feature.options = Some(
                    feature
                        .parameters
                        .split('|')
                        .map(|v| Value::String(v.to_owned()))
                        .collect(),
                );
            }
            "--resolution" => {
                let values = numeric_values(&feature.parameters);
                if feature.parameters.contains('|') {
                    feature.options =
                        Some(values.iter().map(|value| Value::from(*value)).collect());
                    feature.default = numeric_value(default_text);
                } else if let Some(limits) = numeric_range(&feature.parameters) {
                    feature.limits = Some(limits.clone());
                    feature.interval = Some(1.0);
                    let mut options = Vec::new();
                    let mut value = limits[1];
                    while value > limits[0] && options.len() < 32 {
                        options.push(value);
                        value /= 2.0;
                    }
                    options.push(limits[0]);
                    options.sort_by(f64::total_cmp);
                    feature.options = Some(options.into_iter().map(Value::from).collect());
                    feature.default = numeric_value(default_text);
                }
            }
            "-l" | "-t" | "-x" | "-y" | "--page-height" | "--page-width" | "--brightness"
            | "--contrast" => {
                if let Some(mut limits) = numeric_range(&feature.parameters) {
                    if matches!(
                        name.as_str(),
                        "-l" | "-t" | "-x" | "-y" | "--page-height" | "--page-width"
                    ) {
                        limits
                            .iter_mut()
                            .for_each(|value| *value = (*value * 10.0).floor() / 10.0);
                    }
                    feature.limits = Some(limits);
                    feature.interval = Some(step_size(&feature.parameters).unwrap_or(1.0));
                    feature.default = numeric_value(default_text);
                }
            }
            "--ald" => {
                if let Some(options) = feature
                    .parameters
                    .strip_prefix("=(")
                    .and_then(|s| s.strip_suffix(')'))
                {
                    feature.options = Some(
                        options
                            .split('|')
                            .map(|v| Value::String(v.to_owned()))
                            .collect(),
                    );
                }
            }
            _ => {}
        }
        features.insert(name, feature);
    }

    if !features.contains_key("--resolution") {
        return Err("Scanner does not expose a resolution setting".to_owned());
    }

    Ok(Device {
        id: id.to_owned(),
        name: id.to_owned(),
        features,
        string: output.to_owned(),
        settings: DeviceSettings {
            batch_mode: Setting {
                options: batch_modes(),
                default: Value::String("none".to_owned()),
            },
            filters: Setting {
                options: filters(),
                default: Value::Array(Vec::new()),
            },
            pipeline: Setting {
                options: pipeline_options.to_vec(),
                default: Value::String(pipeline_options.first().cloned().unwrap_or_default()),
            },
        },
    })
}

fn number_or_string(text: &str) -> Value {
    text.parse::<f64>()
        .map(Value::from)
        .unwrap_or_else(|_| Value::String(text.to_owned()))
}

fn numeric_value(text: &str) -> Value {
    number_or_string(text)
}

fn numeric_values(text: &str) -> Vec<f64> {
    text.split('|')
        .filter_map(|part| {
            let number = part.trim().trim_end_matches("dpi").trim_end_matches("mm");
            number.parse::<f64>().ok()
        })
        .collect()
}

fn numeric_range(text: &str) -> Option<Vec<f64>> {
    let range = Regex::new(r"(-?\d+(?:\.\d+)?)\s*\.\.\s*(-?\d+(?:\.\d+)?)").ok()?;
    let captures = range.captures(text)?;
    Some(vec![captures[1].parse().ok()?, captures[2].parse().ok()?])
}

fn step_size(text: &str) -> Option<f64> {
    let step = Regex::new(r"\(in steps of (\d+(?:\.\d+)?)\)").ok()?;
    step.captures(text)?.get(1)?.as_str().parse().ok()
}

pub fn batch_modes() -> Vec<String> {
    ["none", "manual", "auto", "auto-collate-standard"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

pub fn filters() -> Vec<String> {
    [
        "filter.auto-contrast",
        "filter.auto-level",
        "filter.threshold",
        "filter.blur",
        "filter.more-contrast",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

pub fn pipeline_options() -> Vec<String> {
    [
        "JPG | @:pipeline.high-quality",
        "JPG | @:pipeline.medium-quality",
        "JPG | @:pipeline.low-quality",
        "PNG",
        "TIF | @:pipeline.uncompressed",
        "TIF | @:pipeline.lzw-compressed",
        "PDF (TIF | @:pipeline.uncompressed)",
        "PDF (TIF | @:pipeline.lzw-compressed)",
        "PDF (JPG | @:pipeline.high-quality)",
        "PDF (JPG | @:pipeline.medium-quality)",
        "PDF (JPG | @:pipeline.low-quality)",
        "@:pipeline.ocr | PDF (JPG | @:pipeline.high-quality)",
        "@:pipeline.ocr | @:pipeline.text-file",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
