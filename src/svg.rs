//! SVG generation for neofetch-style profile cards.
//!
//! Layout: ASCII art (left) | stats column (right)
//!
//! The stats column uses CSS classes for theming:
//! - `.key` - stat labels
//! - `.value` - stat values  
//! - `.cc` - connector characters (dots, parens)
//! - `.addColor` / `.delColor` - LOC additions/deletions

use crate::config::Config;
use crate::theme;
use std::collections::HashMap;
use std::fs;

const TEXT_TOP: i32 = 30;
const LINE_H: i32 = 20;
const ASCII_X: f32 = 15.0;
const COL_GAP: f32 = 10.0;
const RIGHT_PAD: f32 = 30.0;
const CHAR_W: f32 = 9.6;
const MIN_WIDTH: usize = 50;

pub enum OutputMode {
    Dark,
    Light,
}

pub struct Stats {
    pub repos: u32,
    pub stars: u32,
    pub followers: u32,
    pub commits_total: u32,
    pub contributed_repos: u32,
    pub loc_add: i64,
    pub loc_del: i64,
    pub loc_total: i64,
}

/// Represents a line in the right stats column.
///
/// Special variants (Loc, Repos) have custom rendering with colored sub-parts.
enum Line {
    Header(String),
    Blank,
    Stat { key: String, value: String },
    Loc { total: i64, add: i64, del: i64 },
    Repos { count: u32, contributed: u32 },
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn dots(key: &str, value: &str, width: usize) -> String {
    let need = width.saturating_sub(key.len() + 2 + value.len());
    match need {
        0 => String::new(),
        1 => " ".into(),
        2 => ". ".into(),
        n => ".".repeat(n),
    }
}

fn header(text: &str, width: usize) -> String {
    let dashes = width.saturating_sub(text.len()) + 3;
    format!("{text} {}", "-".repeat(dashes))
}

fn age_string(birthday: &str) -> String {
    use jiff::{Unit, Zoned};

    let Ok(broken) = jiff::fmt::strtime::parse("%Y-%m-%d", birthday) else {
        return "Unknown".into();
    };
    let Ok(birth) = broken.to_date() else {
        return "Unknown".into();
    };

    let today = Zoned::now().date();
    let Ok(span) = today.since((Unit::Year, birth)) else {
        return "Unknown".into();
    };

    let years = span.get_years();
    let months = span.get_months();
    let days = span.get_days();

    format!(
        "{} year{}, {} month{}, {} day{}",
        years,
        if years == 1 { "" } else { "s" },
        months,
        if months == 1 { "" } else { "s" },
        days,
        if days == 1 { "" } else { "s" }
    )
}

fn group_by_color(line: &str, map: &HashMap<char, String>) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let color = map.get(&chars[i]).cloned();
        let mut seg = String::from(chars[i]);

        while i + 1 < chars.len() && map.get(&chars[i + 1]) == color.as_ref() {
            seg.push(chars[i + 1]);
            i += 1;
        }

        out.push((seg, color));
        i += 1;
    }
    out
}

fn build_ascii(ascii: &str, map: &HashMap<char, String>) -> (String, usize) {
    let mut out = String::new();
    let mut max_w = 0;

    for (i, line) in ascii.lines().enumerate() {
        let y = TEXT_TOP + (i as i32) * LINE_H;
        max_w = max_w.max(line.len());

        if line.is_empty() {
            out.push_str(&format!(r#"<tspan x="{ASCII_X}" y="{y}"> </tspan>"#));
        } else {
            for (j, (text, color)) in group_by_color(line, map).into_iter().enumerate() {
                let esc = escape_xml(&text);
                if j == 0 {
                    if let Some(c) = &color {
                        out.push_str(&format!(
                            r#"<tspan x="{ASCII_X}" y="{y}" fill="{c}">{esc}</tspan>"#
                        ));
                    } else {
                        out.push_str(&format!(r#"<tspan x="{ASCII_X}" y="{y}">{esc}</tspan>"#));
                    }
                } else if let Some(c) = &color {
                    out.push_str(&format!(r#"<tspan fill="{c}">{esc}</tspan>"#));
                } else {
                    out.push_str(&format!("<tspan>{esc}</tspan>"));
                }
            }
        }
        out.push('\n');
    }
    (out, max_w)
}

fn build_lines(stats: &Stats, cfg: &Config, width: usize) -> Vec<Line> {
    let uptime = age_string(&cfg.birthday);

    vec![
        Line::Header(header(&cfg.name, width)),
        Line::Stat {
            key: "OS".into(),
            value: cfg.system.os.clone(),
        },
        Line::Stat {
            key: "Uptime".into(),
            value: uptime,
        },
        Line::Stat {
            key: "Host".into(),
            value: cfg.system.host.clone(),
        },
        Line::Stat {
            key: "Kernel".into(),
            value: cfg.system.kernel.clone(),
        },
        Line::Stat {
            key: "IDE".into(),
            value: cfg.system.ide.clone(),
        },
        Line::Blank,
        Line::Stat {
            key: "Languages.Programming".into(),
            value: cfg.languages.programming.clone(),
        },
        Line::Stat {
            key: "Languages.Computer".into(),
            value: cfg.languages.computer.clone(),
        },
        Line::Stat {
            key: "Languages.Real".into(),
            value: cfg.languages.real.clone(),
        },
        Line::Blank,
        Line::Stat {
            key: "Hobbies.Software".into(),
            value: cfg.hobbies.software.clone(),
        },
        Line::Stat {
            key: "Hobbies.Hardware".into(),
            value: cfg.hobbies.hardware.clone(),
        },
        Line::Blank,
        Line::Header(header(&cfg.headers.contact, width)),
        Line::Stat {
            key: "Email.Personal".into(),
            value: cfg.contact.personal_email.clone(),
        },
        Line::Stat {
            key: "Email.Work".into(),
            value: cfg.contact.work_email.clone(),
        },
        Line::Stat {
            key: "LinkedIn".into(),
            value: cfg.contact.linkedin.clone(),
        },
        Line::Blank,
        Line::Header(header(&cfg.headers.github_stats, width)),
        Line::Repos {
            count: stats.repos,
            contributed: stats.contributed_repos,
        },
        Line::Stat {
            key: "Stars".into(),
            value: stats.stars.to_string(),
        },
        Line::Stat {
            key: "Commits".into(),
            value: stats.commits_total.to_string(),
        },
        Line::Stat {
            key: "Followers".into(),
            value: stats.followers.to_string(),
        },
        Line::Loc {
            total: stats.loc_total,
            add: stats.loc_add,
            del: stats.loc_del,
        },
    ]
}

fn calc_width(lines: &[Line]) -> usize {
    let mut max = 0;
    for line in lines {
        let len = match line {
            Line::Stat { key, value } => key.len() + 2 + value.len(),
            Line::Repos { count, contributed } => {
                format!("Repos: {} (Contributed: {})", count, contributed).len()
            }
            Line::Loc { total, add, del } => {
                format!("LoC on GitHub: {} ( {}++, {}-- )", total, add, del).len()
            }
            Line::Header(h) => h.len(),
            Line::Blank => 0,
        };
        max = max.max(len);
    }
    max.max(MIN_WIDTH)
}

fn render_lines(lines: &[Line], x: f32, width: usize) -> (String, f32) {
    let mut out = String::new();

    for (i, line) in lines.iter().enumerate() {
        let y = TEXT_TOP + (i as i32) * LINE_H;

        match line {
            Line::Blank => {}
            Line::Header(text) => {
                out.push_str(&format!(
                    r#"<tspan x="{x}" y="{y}">{}</tspan>"#,
                    escape_xml(text)
                ));
            }
            Line::Stat { key, value } => {
                let d = dots(key, value, width);
                out.push_str(&format!(
                    r#"<tspan x="{x}" y="{y}" class="cc">. </tspan><tspan class="key">{}: </tspan><tspan class="cc">{d}</tspan><tspan class="value">{}</tspan>"#,
                    escape_xml(key), escape_xml(value)
                ));
            }
            Line::Repos { count, contributed } => {
                let val = format!("{count} (Contributed: {contributed})");
                let d = dots("Repos", &val, width);
                out.push_str(&format!(
                    r#"<tspan x="{x}" y="{y}" class="cc">. </tspan><tspan class="key">Repos: </tspan><tspan class="cc">{d}</tspan><tspan class="value">{val}</tspan>"#
                ));
            }
            Line::Loc { total, add, del } => {
                let val = format!("{} ( {}++, {}-- )", total, add, del);
                let d = dots("LoC on GitHub", &val, width);
                out.push_str(&format!(
                    r#"<tspan x="{x}" y="{y}" class="cc">. </tspan><tspan class="key">LoC on GitHub: </tspan><tspan class="cc">{d}</tspan><tspan class="value">{total}</tspan><tspan class="cc"> ( </tspan><tspan class="addColor">{add}++</tspan><tspan class="cc">, </tspan><tspan class="delColor">{del}--</tspan><tspan class="cc"> )</tspan>"#
                ));
            }
        }
        if !matches!(line, Line::Blank) {
            out.push('\n');
        }
    }

    let h = lines.len() as f32 * LINE_H as f32 + TEXT_TOP as f32;
    (out, h)
}

pub fn generate_svg(stats: &Stats, config: &Config, mode: OutputMode) -> String {
    let palette = theme::get_theme(&config.theme);
    let (bg, text) = match mode {
        OutputMode::Dark => (palette.bg, palette.text),
        OutputMode::Light => ("#ffffff", "#24292f"),
    };

    let mut colors = palette.ascii_color_map();
    for (ch, c) in &config.ascii_colors {
        colors.insert(*ch, c.clone());
    }

    let ascii = fs::read_to_string(&config.ascii_file).unwrap_or_default();
    let (ascii_svg, ascii_w) = build_ascii(&ascii, &colors);
    let ascii_h = ascii.lines().count() as f32 * LINE_H as f32 + TEXT_TOP as f32;
    let ascii_px = ascii_w as f32 * CHAR_W + ASCII_X;

    let lines = build_lines(stats, config, 0);
    let width = calc_width(&lines);
    let lines = build_lines(stats, config, width);
    let (right_svg, right_h) = render_lines(&lines, ascii_px + COL_GAP, width);

    let w = ascii_px + COL_GAP + width as f32 * CHAR_W + RIGHT_PAD;
    let h = ascii_h.max(right_h) + 30.0;

    format!(
        r#"<?xml version='1.0' encoding='UTF-8'?>
<svg xmlns="http://www.w3.org/2000/svg" width="{w}px" height="{h}px" font-family="ConsolasFallback,Consolas,monospace" font-size="16px">
<style>
.key      {{ fill: {key}; }}
.value    {{ fill: {value}; }}
.cc       {{ fill: {cc}; }}
.addColor {{ fill: #3fb950; }}
.delColor {{ fill: #f85149; }}
</style>
<rect width="{w}px" height="{h}px" fill="{bg}" rx="15"/>
<text fill="{text}" xml:space="preserve">
{ascii_svg}</text>
<text fill="{text}">
{right_svg}</text>
</svg>
"#,
        key = palette.key,
        value = palette.value,
        cc = palette.cc
    )
}
