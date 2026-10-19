use std::{collections::BTreeMap, process::Output};

use serde::Serialize;
use tokio::process::Command;

#[derive(Clone)]
pub struct Printer {
    queue: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterOption {
    pub name: String,
    pub label: String,
    pub choices: Vec<String>,
    pub default: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub queue: String,
    pub media: Option<PrinterOption>,
    pub media_type: Option<PrinterOption>,
    pub color: Option<PrinterOption>,
    pub quality: Option<PrinterOption>,
    pub duplex: Option<PrinterOption>,
    pub borderless: Option<PrinterOption>,
}

#[derive(Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub media: Option<String>,
    pub media_type: Option<String>,
    pub color: Option<String>,
    pub quality: Option<String>,
    pub duplex: Option<String>,
    pub borderless: Option<String>,
    pub copies: Option<u8>,
    pub fit_to_page: Option<bool>,
}

impl Printer {
    pub fn from_env() -> Self {
        Self {
            queue: std::env::var("PRINTER_QUEUE")
                .ok()
                .map(|queue| queue.trim().to_owned())
                .filter(|queue| !queue.is_empty()),
        }
    }

    pub async fn capabilities(&self) -> Result<Capabilities, String> {
        let queue = self.resolve_queue().await?;
        let choices = output("lpoptions", &["-p", &queue, "-l"]).await?;
        let defaults = output("lpoptions", &["-p", &queue]).await?;
        let options = parse_options(&String::from_utf8_lossy(&choices.stdout));
        let defaults = parse_defaults(&String::from_utf8_lossy(&defaults.stdout));

        Ok(Capabilities {
            media: option(&options, &defaults, "PageSize", "Media size"),
            media_type: option(&options, &defaults, "MediaType", "Media type"),
            color: option(&options, &defaults, "ColorModel", "Color"),
            quality: option(&options, &defaults, "cupsPrintQuality", "Print quality")
                .or_else(|| option(&options, &defaults, "PrintQuality", "Print quality")),
            duplex: option(&options, &defaults, "Duplex", "Two-sided"),
            borderless: option(&options, &defaults, "Borderless", "Borderless"),
            queue,
        })
    }

    pub async fn submit(&self, path: &str, settings: &Settings) -> Result<String, String> {
        let capabilities = self.capabilities().await?;
        let copies = settings.copies.unwrap_or(1);
        if !(1..=99).contains(&copies) {
            return Err("Copies must be between 1 and 99".to_owned());
        }

        let mut args = vec![
            "-d".to_owned(),
            capabilities.queue.clone(),
            "-n".to_owned(),
            copies.to_string(),
        ];
        for (value, option) in [
            (settings.media.as_deref(), capabilities.media.as_ref()),
            (
                settings.media_type.as_deref(),
                capabilities.media_type.as_ref(),
            ),
            (settings.color.as_deref(), capabilities.color.as_ref()),
            (settings.quality.as_deref(), capabilities.quality.as_ref()),
            (settings.duplex.as_deref(), capabilities.duplex.as_ref()),
            (
                settings.borderless.as_deref(),
                capabilities.borderless.as_ref(),
            ),
        ] {
            if let Some(value) = value {
                let option = option
                    .ok_or_else(|| "This printer does not support that setting".to_owned())?;
                if !option.choices.iter().any(|choice| choice == value) {
                    return Err(format!("Unsupported {} value", option.label));
                }
                args.extend(["-o".to_owned(), format!("{}={value}", option.name)]);
            }
        }
        args.extend([
            "-o".to_owned(),
            format!(
                "print-scaling={}",
                if settings.fit_to_page.unwrap_or(false) {
                    "fit"
                } else {
                    "none"
                }
            ),
        ]);
        args.push(path.to_owned());

        let borrowed = args.iter().map(String::as_str).collect::<Vec<_>>();
        let output = output("lp", &borrowed).await?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    async fn resolve_queue(&self) -> Result<String, String> {
        if let Some(queue) = &self.queue {
            if queue.len() > 127
                || !queue
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c))
            {
                return Err("PRINTER_QUEUE has an invalid queue name".to_owned());
            }
            return Ok(queue.clone());
        }

        let output = output("lpstat", &["-d"]).await?;
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.strip_prefix("system default destination:"))
            .map(str::trim)
            .filter(|queue| !queue.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| "No default printer is configured; set PRINTER_QUEUE".to_owned())
    }
}

fn option(
    options: &BTreeMap<String, (String, Vec<String>, Option<String>)>,
    defaults: &BTreeMap<String, String>,
    name: &str,
    fallback_label: &str,
) -> Option<PrinterOption> {
    let (label, choices, ppd_default) = options.get(name)?;
    Some(PrinterOption {
        name: name.to_owned(),
        label: if label.is_empty() {
            fallback_label.to_owned()
        } else {
            label.clone()
        },
        choices: choices.clone(),
        default: defaults
            .get(name)
            .cloned()
            .filter(|value| choices.contains(value))
            .or_else(|| ppd_default.clone()),
    })
}

fn parse_options(text: &str) -> BTreeMap<String, (String, Vec<String>, Option<String>)> {
    text.lines()
        .filter_map(|line| {
            let (description, choices) = line.split_once(':')?;
            let (name, label) = description
                .split_once('/')
                .unwrap_or((description, description));
            let mut default = None;
            let values = choices
                .split_whitespace()
                .map(|value| {
                    if let Some(value) = value.strip_prefix('*') {
                        default = Some(value.to_owned());
                        value.to_owned()
                    } else {
                        value.to_owned()
                    }
                })
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();
            (!name.is_empty() && !values.is_empty())
                .then(|| (name.to_owned(), (label.to_owned(), values, default)))
        })
        .collect()
}

fn parse_defaults(text: &str) -> BTreeMap<String, String> {
    text.split_whitespace()
        .filter_map(|part| {
            let (name, value) = part.split_once('=')?;
            Some((name.to_owned(), value.to_owned()))
        })
        .collect()
}

async fn output(program: &str, args: &[&str]) -> Result<Output, String> {
    let result = Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|error| format!("Unable to run {program}: {error}"))?;
    if !result.status.success() {
        let error = String::from_utf8_lossy(&result.stderr).trim().to_owned();
        return Err(if error.is_empty() {
            format!("{program} exited with {}", result.status)
        } else {
            error
        });
    }
    Ok(result)
}
