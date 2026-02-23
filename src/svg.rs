use crate::age;
use crate::config::Config;
use std::collections::HashMap;
use std::fs;

// Layout constants for SVG positioning
const TEXT_TOP_MARGIN_PX: i32 = 30;
const LINE_SPACING_PX: i32 = 20;
const ASCII_LEFT_MARGIN_PX: f32 = 15.0;
const SPACE_BETWEEN_COLUMNS_PX: f32 = 10.0;
const RIGHT_MARGIN_PX: f32 = 30.0;
const MONOCHAR_WIDTH_PX: f32 = 9.6;
const MIN_RIGHT_COLUMN_WIDTH_CHARS: usize = 50;

#[derive(Clone, Copy)]
pub enum Theme {
    Dark,
    Light,
}

pub struct ThemeColors {
    pub bg: &'static str,
    pub text: &'static str,
    pub key: &'static str,
    pub value: &'static str,
    pub cc: &'static str,
}

impl Theme {
    pub fn colors(self) -> ThemeColors {
        match self {
            Theme::Dark => ThemeColors {
                bg: "#161b22",
                text: "#c9d1d9",
                key: "#ffa657",
                value: "#a5d6ff",
                cc: "#616e7f",
            },
            Theme::Light => ThemeColors {
                bg: "#ffffff",
                text: "#24292f",
                key: "#d73a49",
                value: "#0366d6",
                cc: "#6a737d",
            },
        }
    }
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

// Utilities for building SVG content

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn build_stat_row(key: &str, value: &str, align_width: usize) -> (String, String, String) {
    let key_part = format!("{key}: ");
    let base_len = key_part.len() + value.len();
    let available = align_width.saturating_sub(base_len);

    let dots = match available {
        0 => "".to_string(),
        1 => " ".to_string(),
        2 => ". ".to_string(),
        n => ".".repeat(n),
    };

    (key_part, dots, value.to_string())
}

fn build_header_line(label: &str, align_width: usize) -> String {
    let base = format!("{label} ");
    let dash_count = align_width.saturating_sub(base.len()) + 2;
    format!("{base}{}", "-".repeat(dash_count))
}

fn build_ascii_tspans(ascii: &str, color_map: &HashMap<char, String>) -> (String, usize) {
    let mut out = String::new();
    let mut max_width = 0;

    for (i, line) in ascii.lines().enumerate() {
        let y = TEXT_TOP_MARGIN_PX + (i as i32) * LINE_SPACING_PX;
        max_width = max_width.max(line.len());

        if line.is_empty() {
            out.push_str(&format!(
                "<tspan x=\"{ASCII_LEFT_MARGIN_PX}\" y=\"{y}\"> </tspan>\n"
            ));
            continue;
        }

        let segments = group_by_color(line, color_map);
        let mut first = true;
        for (text, color) in segments {
            let escaped = escape_xml(&text);
            if first {
                if let Some(c) = color {
                    out.push_str(&format!(
                        "<tspan x=\"{ASCII_LEFT_MARGIN_PX}\" y=\"{y}\" fill=\"{c}\">{escaped}</tspan>"
                    ));
                } else {
                    out.push_str(&format!(
                        "<tspan x=\"{ASCII_LEFT_MARGIN_PX}\" y=\"{y}\">{escaped}</tspan>"
                    ));
                }
                first = false;
            } else if let Some(c) = color {
                out.push_str(&format!("<tspan fill=\"{c}\">{escaped}</tspan>"));
            } else {
                out.push_str(&format!("<tspan>{escaped}</tspan>"));
            }
        }
        out.push('\n');
    }

    (out, max_width)
}

fn group_by_color(line: &str, color_map: &HashMap<char, String>) -> Vec<(String, Option<String>)> {
    let mut segments: Vec<(String, Option<String>)> = Vec::new();
    let chars: Vec<char> = line.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let color = color_map.get(&ch).cloned();
        let mut segment = String::new();
        segment.push(ch);

        while i + 1 < chars.len() {
            let next_ch = chars[i + 1];
            let next_color = color_map.get(&next_ch).cloned();
            if next_color == color {
                segment.push(next_ch);
                i += 1;
            } else {
                break;
            }
        }

        segments.push((segment, color));
        i += 1;
    }

    segments
}

fn build_right_column(
    stats: &Stats,
    config: &Config,
    ascii_width_px: f32,
    ascii_height_px: f32,
) -> (String, f32, f32) {
    let os_value = config.system.os.clone();
    let uptime_value =
        age::age_from_birthday(&config.birthday).unwrap_or_else(|_| "Unknown".to_string());
    let host_value = config.system.host.clone();
    let kernel_value = config.system.kernel.clone();
    let ide_value = config.system.ide.clone();

    let lang_prog_value = config.languages.programming.clone();
    let lang_comp_value = config.languages.computer.clone();
    let lang_real_value = config.languages.real.clone();

    let hobby_soft_value = config.hobbies.software.clone();
    let hobby_hard_value = config.hobbies.hardware.clone();

    let contact_personal_value = config.contact.personal_email.clone();
    let contact_work_value = config.contact.work_email.clone();
    let contact_linkedin_value = config.contact.linkedin.clone();

    let repos_value = stats.repos.to_string();
    let contrib_value = stats.contributed_repos.to_string();
    let repos_fake = format!("{repos_value} (Contributed: {contrib_value}) ");
    let stars_value = stats.stars.to_string();
    let commits_value = stats.commits_total.to_string();
    let followers_value = stats.followers.to_string();

    let loc_total_value = stats.loc_total.to_string();
    let loc_add_value = stats.loc_add.to_string();
    let loc_del_value = stats.loc_del.to_string();

    let loc_fake = format!(
        " {} ( {}++ , {}--) ",
        loc_total_value, loc_add_value, loc_del_value
    );

    let rows_for_width: Vec<(&str, &String)> = vec![
        ("OS", &os_value),
        ("Uptime", &uptime_value),
        ("Host", &host_value),
        ("Kernel", &kernel_value),
        ("IDE", &ide_value),
        ("Languages.Programming", &lang_prog_value),
        ("Languages.Computer", &lang_comp_value),
        ("Languages.Real", &lang_real_value),
        ("Hobbies.Software", &hobby_soft_value),
        ("Hobbies.Hardware", &hobby_hard_value),
        ("Email.Personal", &contact_personal_value),
        ("Email.Work", &contact_work_value),
        ("LinkedIn", &contact_linkedin_value),
        ("Repos", &repos_fake),
        ("Stars", &stars_value),
        ("Commits", &commits_value),
        ("Followers", &followers_value),
        ("LoC on GitHub", &loc_fake),
    ];

    let mut align_width = rows_for_width
        .iter()
        .map(|(k, v)| k.len() + 2 + v.len())
        .max()
        .unwrap_or(0);

    align_width = align_width.max(MIN_RIGHT_COLUMN_WIDTH_CHARS);

    macro_rules! row {
        ($k:expr, $v:expr) => {
            build_stat_row($k, $v, align_width)
        };
    }

    // Individual rows
    let (os_k, os_d, os_v) = row!("OS", &os_value);
    let (up_k, up_d, up_v) = row!("Uptime", &uptime_value);
    let (ho_k, ho_d, ho_v) = row!("Host", &host_value);
    let (ke_k, ke_d, ke_v) = row!("Kernel", &kernel_value);
    let (id_k, id_d, id_v) = row!("IDE", &ide_value);

    let (lp_k, lp_d, lp_v) = row!("Languages.Programming", &lang_prog_value);
    let (lc_k, lc_d, lc_v) = row!("Languages.Computer", &lang_comp_value);
    let (lr_k, lr_d, lr_v) = row!("Languages.Real", &lang_real_value);

    let (hs_k, hs_d, hs_v) = row!("Hobbies.Software", &hobby_soft_value);
    let (hh_k, hh_d, hh_v) = row!("Hobbies.Hardware", &hobby_hard_value);

    let (ep_k, ep_d, ep_v) = row!("Email.Personal", &contact_personal_value);
    let (ew_k, ew_d, ew_v) = row!("Email.Work", &contact_work_value);
    let (li_k, li_d, li_v) = row!("LinkedIn", &contact_linkedin_value);

    let (re_k, re_d, re_v) = row!("Repos", &repos_fake);
    let (st_k, st_d, st_v) = row!("Stars", &stars_value);
    let (cm_k, cm_d, cm_v) = row!("Commits", &commits_value);
    let (fo_k, fo_d, fo_v) = row!("Followers", &followers_value);
    let (lo_k, lo_d, _) = row!("LoC on GitHub", &loc_fake);

    // Headers
    let h_main = build_header_line(&config.name, align_width);
    let h_contact = build_header_line(&config.headers.contact, align_width);
    let h_stats = build_header_line(&config.headers.github_stats, align_width);

    let dummy = "".to_string();
    enum Line<'a> {
        Header(&'a String),
        Blank,
        Stat {
            k: &'a String,
            d: &'a String,
            v: &'a String,
        },
    }

    let lines: Vec<Line> = vec![
        Line::Header(&h_main),
        Line::Stat {
            k: &os_k,
            d: &os_d,
            v: &os_v,
        },
        Line::Stat {
            k: &up_k,
            d: &up_d,
            v: &up_v,
        },
        Line::Stat {
            k: &ho_k,
            d: &ho_d,
            v: &ho_v,
        },
        Line::Stat {
            k: &ke_k,
            d: &ke_d,
            v: &ke_v,
        },
        Line::Stat {
            k: &id_k,
            d: &id_d,
            v: &id_v,
        },
        Line::Blank,
        Line::Stat {
            k: &lp_k,
            d: &lp_d,
            v: &lp_v,
        },
        Line::Stat {
            k: &lc_k,
            d: &lc_d,
            v: &lc_v,
        },
        Line::Stat {
            k: &lr_k,
            d: &lr_d,
            v: &lr_v,
        },
        Line::Blank,
        Line::Stat {
            k: &hs_k,
            d: &hs_d,
            v: &hs_v,
        },
        Line::Stat {
            k: &hh_k,
            d: &hh_d,
            v: &hh_v,
        },
        Line::Blank,
        Line::Header(&h_contact),
        Line::Stat {
            k: &ep_k,
            d: &ep_d,
            v: &ep_v,
        },
        Line::Stat {
            k: &ew_k,
            d: &ew_d,
            v: &ew_v,
        },
        Line::Stat {
            k: &li_k,
            d: &li_d,
            v: &li_v,
        },
        Line::Blank,
        Line::Header(&h_stats),
        Line::Stat {
            k: &re_k,
            d: &re_d,
            v: &re_v,
        },
        Line::Stat {
            k: &st_k,
            d: &st_d,
            v: &st_v,
        },
        Line::Stat {
            k: &cm_k,
            d: &cm_d,
            v: &cm_v,
        },
        Line::Stat {
            k: &fo_k,
            d: &fo_d,
            v: &fo_v,
        },
        Line::Stat {
            k: &lo_k,
            d: &lo_d,
            v: &dummy,
        },
    ];

    // Render
    let right_height_px = lines.len() as f32 * LINE_SPACING_PX as f32 + TEXT_TOP_MARGIN_PX as f32;
    let right_x = ascii_width_px + SPACE_BETWEEN_COLUMNS_PX;

    let mut right_tspans = String::new();
    for (i, line) in lines.iter().enumerate() {
        let y = TEXT_TOP_MARGIN_PX + (i as i32) * LINE_SPACING_PX;

        match line {
            Line::Blank => {}
            Line::Header(text) => {
                right_tspans.push_str(&format!(
                    r#"<tspan x="{right_x}" y="{y}">{}</tspan>
"#,
                    escape_xml(text)
                ));
            }
            // Perfectly aligned LOC row
            Line::Stat { k, d: _, .. } if k.starts_with("LoC on GitHub") => {
                right_tspans.push_str(&format!(
                    r#"<tspan x="{right_x}" y="{y}" class="cc">. </tspan>
<tspan class="key">{}</tspan>
<tspan class="cc">{}</tspan>
<tspan class="value">{}</tspan>
<tspan class="cc"> ( </tspan>
<tspan class="addColor">{}</tspan><tspan class="addColor">++</tspan>
<tspan class="cc">, </tspan>
<tspan class="delColor">{}</tspan><tspan class="delColor">--</tspan>
<tspan class="cc"> )</tspan>
"#,
                    escape_xml(&lo_k),
                    escape_xml(&lo_d),
                    loc_total_value,
                    loc_add_value,
                    loc_del_value
                ));
            }
            Line::Stat { k, d, .. } if k.starts_with("Repos") => {
                right_tspans.push_str(&format!(
                    r#"<tspan x="{right_x}" y="{y}" class="cc">. </tspan>
<tspan class="key">{}</tspan>
<tspan class="cc">{}</tspan>
<tspan class="value">{} (Contributed: {})</tspan>
"#,
                    escape_xml(k),
                    escape_xml(d),
                    repos_value,
                    contrib_value
                ));
            }

            // Normal rows
            Line::Stat { k, d, v } => {
                right_tspans.push_str(&format!(
                    r#"<tspan x="{right_x}" y="{y}" class="cc">. </tspan>
<tspan class="key">{}</tspan><tspan class="cc">{}</tspan><tspan class="value">{}</tspan>
"#,
                    escape_xml(k),
                    escape_xml(d),
                    escape_xml(v)
                ));
            }
        }
    }

    let content_width = right_x + (align_width as f32) * MONOCHAR_WIDTH_PX + RIGHT_MARGIN_PX;
    let content_height = ascii_height_px.max(right_height_px) + 30.0;

    (right_tspans, content_width, content_height)
}

pub fn generate_svg(stats: &Stats, config: &Config, theme: Theme) -> String {
    let colors = theme.colors();

    let ascii = fs::read_to_string(&config.ascii_file).unwrap_or_default();
    let (ascii_tspans, ascii_chars_wide) = build_ascii_tspans(&ascii, &config.ascii_colors);
    let ascii_lines = ascii.lines().count();
    let ascii_width_px = ascii_chars_wide as f32 * MONOCHAR_WIDTH_PX + ASCII_LEFT_MARGIN_PX;
    let ascii_height_px = ascii_lines as f32 * LINE_SPACING_PX as f32 + TEXT_TOP_MARGIN_PX as f32;

    let (right_tspans, w, h) = build_right_column(stats, config, ascii_width_px, ascii_height_px);

    format!(
        r#"<?xml version='1.0' encoding='UTF-8'?>
<svg xmlns="http://www.w3.org/2000/svg"
     width="{w}px" height="{h}px"
     font-family="ConsolasFallback,Consolas,monospace"
     font-size="16px">

<style>
.key      {{ fill: {key}; }}
.value    {{ fill: {value}; }}
.cc       {{ fill: {cc}; }}
.addColor {{ fill: #3fb950; }}
.delColor {{ fill: #f85149; }}
</style>

<rect width="{w}px" height="{h}px" fill="{bg}" rx="15"/>

<!-- LEFT ASCII -->
<text fill="{text}" xml:space="preserve">
{ascii}
</text>

<!-- RIGHT COLUMN -->
<text fill="{text}">
{right}
</text>

</svg>
"#,
        w = w,
        h = h,
        bg = colors.bg,
        text = colors.text,
        key = colors.key,
        value = colors.value,
        cc = colors.cc,
        ascii = ascii_tspans,
        right = right_tspans
    )
}
