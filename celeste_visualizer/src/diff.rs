use celeste_save_data_rs::save_data::{ SaveData, MapCode };
use celeste_save_data_rs::map_data::GameData;
use std::collections::HashMap;

struct SaveDataDiff {
    stats_diffs: Vec<(MapData, StatsDiff)>,
}

impl SaveDataDiff {
    fn new() -> Self {
        Self {
            stats_diffs: Vec::new(),
        }
    }
    fn create_diff(game_data: &GameData, before: &SaveData, after: &SaveData) -> Self {
        let mut diff = Self::new();
        for level in game_data.levels() {
            for map_data in game_data.get_level_data(level).unwrap().maps() {
                let stats_diff = match (before.map_stats.get(&map_data.code), after.map_stats.get(&map_data.code)) {
                    (None, None) => StatsDiff::Same,
                    (Some(_), None) => StatsDiff::BeforeOnly,
                    (None, Some(_)) => StatsDiff::AfterOnly,
                    (Some(before), Some(after)) => {
                        let strawberries = {
                            let b = before.total_strawberries();
                            let a = after.total_strawberries();
                            if b == a { DiffParam::Same }
                            else if b < a { DiffParam::Normal(format!("+{}", a - b)) }
                            else { DiffParam::Outlier(format!("-{}", b - a)) }
                        };
                        let best_deaths = {
                            let bsr = before.single_run_completed;
                            let asr = after.single_run_completed;
                            if bsr && asr {
                                let b = before.best_deaths;
                                let a = after.best_deaths;
                                if b == a { DiffParam::Same }
                                else if b > a { DiffParam::Normal(format!("-{}", b - a)) }
                                else { DiffParam::Outlier(format!("+{}", a - b)) }
                            }
                            else if !bsr && !asr {
                                DiffParam::Same
                            }
                            else if !bsr && asr {
                                DiffParam::Normal("new".to_string())
                            }
                            else {
                                DiffParam::Outlier("degrate".to_string())
                            }
                        };
                        let deaths = {
                            let b = before.deaths;
                            let a = after.deaths;
                            if b == a { DiffParam::Same }
                            else if b < a { DiffParam::Normal(format!("+{}", a - b)) }
                            else { DiffParam::Outlier(format!("-{}", b - a)) }
                        };
                        let clr = {
                            let bsr = before.single_run_completed;
                            let asr = after.single_run_completed;
                            if bsr && asr {
                                let b = before.best_time;
                                let a = after.best_time;
                                if b == a { DiffParam::Same }
                                else if b > a { DiffParam::Normal(format!("-{}", b - a)) }
                                else { DiffParam::Outlier(format!("+{}", a - b)) }
                            }
                            else if !bsr && !asr {
                                let bsr = before.completed;
                                let asr = after.completed;
                                if (bsr && asr) || (!bsr && !asr) {
                                    let b = before.time_played;
                                    let a = after.time_played;
                                    if b == a { DiffParam::Same }
                                    else if b > a { DiffParam::Normal(format!("-{}", b - a)) }
                                    else { DiffParam::Outlier(format!("+{}", a - b)) }
                                }
                                else if !bsr && asr {
                                    DiffParam::Normal("new".to_string())
                                }
                                else {
                                    DiffParam::Outlier("degrate".to_string())
                                }
                            }
                            else if !bsr && asr {
                                DiffParam::Normal("new".to_string())
                            }
                            else {
                                DiffParam::Outlier("degrate".to_string())
                            }
                        };
                        let fc = {
                            let bsr = before.full_clear;
                            let asr = after.full_clear;
                            if bsr && asr {
                                let b = before.best_full_clear_time;
                                let a = after.best_full_clear_time;
                                if b == a { DiffParam::Same }
                                else if b > a { DiffParam::Normal(format!("-{}", b - a)) }
                                else { DiffParam::Outlier(format!("+{}", a - b)) }
                            }
                            else if !bsr && !asr {
                                DiffParam::Same
                            }
                            else if !bsr && asr {
                                DiffParam::Normal("new".to_string())
                            }
                            else {
                                DiffParam::Outlier("degrate".to_string())
                            }
                        };
                        StatsDiff::Diff { strawberries, best_deaths, deaths, clr, fc }.same_check()
                    }
                };
                if stats_diff != StatsDiff::Same {
                    diff.stats_diffs.push((map_data, stats_diff));
                }
            }
        }
        diff
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatsDiff {
    Same,
    AfterOnly,
    BeforeOnly,
    Diff {
        strawberries: DiffParam,
        best_deaths: DiffParam,
        deaths: DiffParam,
        clr: DiffParam,
        fc: DiffParam,
    }
}

impl StatsDiff {
    pub fn same_check(self) -> Self {
        if let StatsDiff::Diff { strawberries, best_deaths, deaths, clr, fc } = self {
            if strawberries == DiffParam::Same &&
                best_deaths == DiffParam::Same &&
                deaths == DiffParam::Same &&
                clr == DiffParam::Same &&
                fc == DiffParam::Same {
                    StatsDiff::Same
                }
            else {
                StatsDiff::Diff { strawberries, best_deaths, deaths, clr, fc }
            }
        }
        else {
            self
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffParam {
    Same,
    Normal(String),
    Outlier(String),
}

impl Default for DiffParam {
    fn default() -> Self {
        DiffParam::Same
    }
}


use svg_vis::chart::Chart;
use svg_vis::attribute::*;
use svg_vis::literal::Color;
use svg_vis::element::{ Text, Path };
use celeste_save_data_rs::map_data::MapData;
use resvg::usvg::{ fontdb, Tree, TreeParsing, TreeTextToPath };
use resvg::tiny_skia::Pixmap;
use resvg::render;

pub fn diff_svg_chart(game_data: &GameData, before: &SaveData, after: &SaveData, lang: &str) -> (Chart, i64, i64)
{
    let diff = SaveDataDiff::create_diff(game_data, before, after);
    let map_num = diff.stats_diffs.len();

    let margin = 30;
    let row_height = 40;
    let font_size = 25;
    let col_widths = vec![350, 80, 80, 80, 220, 220];
    let col_acc = col_widths.iter().fold(vec![0], |mut v, e| { v.push(v[v.len() - 1] + e); v });
    let chart_width = col_acc[col_acc.len() - 1];
    let chart_height = row_height * (map_num as i64 * 2 + 1);
    let mut chart = Chart::new(-margin, -margin, chart_width + margin * 2, chart_height + margin * 2i64);
    {
        let bg = Path::new()
            .line_rel(chart_width, 0)
            .line_rel(0, chart_height)
            .line_rel(-chart_width, 0)
            .close()
            .fill(Color::from_name("white"));
      chart = chart.draw(bg, 0, 0);
    }

    let centered_text_box = |s: &str| {
        Text::new()
            .set_text(s)
            .font_size(font_size)
            .text_anchor(text_anchor::TextAnchorValue::Middle)
            .dominant_baseline(dominant_baseline::DominantBaselineValue::Central)
    };
    {
        for (i, text) in ["chapter", "SB", "best", "deaths", "CLR", "FC"].into_iter().enumerate() {
            chart = chart
                .draw(centered_text_box(text), col_acc[i] + col_widths[i] / 2, row_height / 2);
        }
    }

    let color255 = |r: f32, g:f32, b:f32| {
        Color::from_rgb(r/255.0, g/255.0, b/255.0)
    };

    let sides = ["A", "B", "C"];

    for i in 0..map_num {
        let color = color255(222.0, 226.0, 230.0);
        let path = Path::new()
            .line_rel(chart_width, 0)
            .stroke_width(1)
            .stroke(color);
        chart = chart.draw(path, 0, row_height * (i as i64 * 2 + 1))
    }
    for (i, (MapData { code, name }, stats_diff)) in diff.stats_diffs.into_iter().enumerate() {
        for (k, sd) in [before, after].into_iter().enumerate() {
            match sd.map_stats.get(&code) {
                None => {
                    let elems = vec![format!("{}-{}", name.try_local_name(lang), sides[code.side]), "-".to_string(), "-".to_string(), "-".to_string(), "-".to_string(), "-".to_string()];
                    for (j, text) in elems.into_iter().enumerate() {
                        if j == 0 {
                            if k == 0 {
                                chart = chart.draw(centered_text_box(&text).text_anchor(text_anchor::TextAnchorValue::Start), col_acc[j], row_height / 2 + row_height * (i as i64 + 1));
                            }
                        }
                        else {
                            chart = chart.draw(centered_text_box(&text), col_acc[j] + col_widths[j] / 2, row_height / 2 + row_height * (i as i64 * 2 + k as i64 + 1));
                        }
                    }
                }
                Some(stats) => {
                    let ch_text = centered_text_box(&format!("{}-{}", name.try_local_name(lang), sides[code.side])).text_anchor(text_anchor::TextAnchorValue::Start);

                    let sb_text = centered_text_box(&stats.total_strawberries().to_string());

                    let best_str = if stats.single_run_completed { stats.best_deaths.to_string() } else { "-".to_string() };
                    let best_bg_color = 
                        if best_str == "0" {
                            color255(255.0, 236.0, 163.0)
                        }
                        else {
                            color255(255.0, 255.0, 255.0)
                        };
                    let best_bg = Path::new()
                        .line_rel(col_widths[2], 0)
                        .line_rel(0, row_height)
                        .line_rel(-col_widths[2], 0)
                        .close()
                        .fill(best_bg_color);
                    let best_text = centered_text_box(&best_str);

                    let deaths_text = centered_text_box(&stats.deaths.to_string());

                    let clr_str = 
                        if stats.single_run_completed { stats.best_time.to_string() }
                        else if stats.completed { format!("[{}]", stats.time_played.to_string()) }
                        else { format!("({})", stats.time_played.to_string()) };
                    let clr_bg_color = 
                        if stats.single_run_completed {
                            color255(255.0, 236.0, 163.0)
                        }
                        else if stats.completed {
                            color255(252.0, 195.0, 50.0)
                        }
                        else {
                            color255(255.0, 255.0, 255.0)
                        };
                    let clr_bg = Path::new()
                        .line_rel(col_widths[4], 0)
                        .line_rel(0, row_height)
                        .line_rel(-col_widths[4], 0)
                        .close()
                        .fill(clr_bg_color);
                    let clr_text = centered_text_box(&clr_str);

                    let fc_str =
                        if stats.full_clear { stats.best_full_clear_time.to_string() }
                        else { "-".to_string() };
                    let fc_bg_color = 
                        if stats.full_clear {
                            color255(255.0, 236.0, 163.0)
                        }
                        else {
                            color255(255.0, 255.0, 255.0)
                        };
                    let fc_bg = Path::new()
                        .line_rel(col_widths[5], 0)
                        .line_rel(0, row_height)
                        .line_rel(-col_widths[5], 0)
                        .close()
                        .fill(fc_bg_color);
                    let fc_text = centered_text_box(&fc_str);
                    let row_start = row_height * (i as i64 * 2 + k as i64 + 1);
                    let row_center = row_height * (i as i64 * 2 + k as i64 + 1) + row_height / 2;
                    if k == 0 {
                        chart = chart
                            .draw(ch_text, col_acc[0], row_center)
                    }
                    chart = chart
                        .draw(sb_text, col_acc[1] + col_widths[1] / 2, row_center)
                        .draw(best_bg, col_acc[2], row_start)
                        .draw(best_text, col_acc[2] + col_widths[2] / 2, row_center)
                        .draw(deaths_text, col_acc[3] + col_widths[3] / 2, row_center)
                        .draw(clr_bg, col_acc[4], row_start)
                        .draw(clr_text, col_acc[4] + col_widths[4] / 2, row_center)
                        .draw(fc_bg, col_acc[5], row_start)
                        .draw(fc_text, col_acc[5] + col_widths[5] / 2, row_center)
                }
            }
        }
        let diff_rect_width = 4;
        let diff_rect_mergin = 2;
        let diff_rect_font_size = 15;

        let diff_rect_text = |s: &str| {
            Text::new()
                .set_text(s)
                .font_size(diff_rect_font_size)
                .text_anchor(text_anchor::TextAnchorValue::End)
                .dominant_baseline(dominant_baseline::DominantBaselineValue::Middle)
        };
        match stats_diff {
            StatsDiff::Same => {}
            StatsDiff::BeforeOnly => {
                let rect = Path::new()
                    .line_rel(chart_width - diff_rect_mergin * 2, 0)
                    .line_rel(0, row_height * 2 - diff_rect_mergin * 2)
                    .line_rel(-chart_width + diff_rect_mergin * 2, 0)
                    .close()
                    .fill_opacity(0)
                    .stroke_width(diff_rect_width)
                    .stroke(Color::from_name("blue"));
                chart = chart.draw(rect, diff_rect_mergin, row_height * (i as i64 * 2 + 1) + diff_rect_mergin);
            }
            StatsDiff::AfterOnly => {
                let rect = Path::new()
                    .line_rel(chart_width - diff_rect_mergin * 2, 0)
                    .line_rel(0, row_height * 2 - diff_rect_mergin * 2)
                    .line_rel(-chart_width + diff_rect_mergin * 2, 0)
                    .close()
                    .fill_opacity(0)
                    .stroke_width(diff_rect_width)
                    .stroke(Color::from_name("red"));
                chart = chart.draw(rect, diff_rect_mergin, row_height * (i as i64 * 2 + 1) + diff_rect_mergin);
            }
            StatsDiff::Diff { strawberries, best_deaths, deaths, clr, fc } => {
                let row_start = row_height * (i as i64 * 2 + 1);
                let row_middle = row_height * (i as i64 * 2 + 1 + 1);
                let diff_rect = |chart: Chart, param, idx: usize| {
                    match param {
                        DiffParam::Same => { chart }
                        DiffParam::Normal(s) => {
                            let rect = Path::new()
                                .line_rel(col_widths[idx] - diff_rect_mergin * 2, 0)
                                .line_rel(0, row_height * 2 - diff_rect_mergin * 2)
                                .line_rel(-col_widths[idx] + diff_rect_mergin * 2, 0)
                                .close()
                                .fill_opacity(0)
                                .stroke_width(diff_rect_width)
                                .stroke(Color::from_name("blue"));
                            let text = diff_rect_text(&s).fill("blue");
                            chart
                                .draw(rect, col_acc[idx] + diff_rect_mergin, row_start + diff_rect_mergin)
                                .draw(text, col_acc[idx + 1] - diff_rect_width, row_middle)
                        }
                        DiffParam::Outlier(s) => {
                            let rect = Path::new()
                                .line_rel(col_widths[idx] - diff_rect_mergin * 2, 0)
                                .line_rel(0, row_height * 2 - diff_rect_mergin * 2)
                                .line_rel(-col_widths[idx] + diff_rect_mergin * 2, 0)
                                .close()
                                .fill_opacity(0)
                                .stroke_width(diff_rect_width)
                                .stroke(Color::from_name("red"));
                            let text = diff_rect_text(&s).fill("red");
                            chart
                                .draw(rect, col_acc[idx] + diff_rect_mergin, row_start + diff_rect_mergin)
                                .draw(text, col_acc[idx + 1] - diff_rect_width, row_middle)
                        }
                    }
                };
                chart = diff_rect(chart, strawberries, 1);
                chart = diff_rect(chart, best_deaths, 2);
                chart = diff_rect(chart, deaths, 3);
                chart = diff_rect(chart, clr, 4);
                chart = diff_rect(chart, fc, 5);
            }
        }
    }

    (chart, chart_width, chart_height)
}

pub fn generate_diff_png<P>(game_data: &GameData, before: &SaveData, after: &SaveData, path: P, lang: &str) -> Result<(), String>
    where P: AsRef<std::path::Path>,
{
    let (chart, width, height) = diff_svg_chart(game_data, before, after, lang);
    let mut pixmap = Pixmap::new(width as u32, height as u32).unwrap();
    let option = {
        let mut opt = resvg::usvg::Options::default();
        opt.font_family = "M+ 1c".to_string();
        opt
    };

    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    let mut tree = Tree::from_str(&chart.to_string(), &option)
        .map_err(|e| format!("parse error: {:?}", e))?;
    tree.convert_text(&fontdb);

    render(&tree, resvg::FitTo::Original, resvg::tiny_skia::Transform::identity(), pixmap.as_mut()).unwrap();
    pixmap.save_png(path).map_err(|e| format!("save error {:?}", e))?;
    Ok(())
}
