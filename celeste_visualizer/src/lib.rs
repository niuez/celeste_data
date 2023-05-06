use svg_vis::chart::Chart;
use svg_vis::attribute::*;
use svg_vis::literal::Color;
use svg_vis::element::{ Text, Path };
use celeste_save_data_rs::save_data::SaveData;
use celeste_save_data_rs::map_data::MapData;
use resvg::usvg::{ fontdb, Tree, TreeParsing, TreeTextToPath };
use resvg::tiny_skia::Pixmap;
use resvg::render;

fn generate_svg_chart<MI>(save_data: &SaveData, map_iter: MI) -> (Chart, i64, i64)
    where MI: IntoIterator<Item=MapData>,
          MI::IntoIter: ExactSizeIterator,
{
    let map_iter = map_iter.into_iter();
    let map_num = map_iter.len();
    let margin = 30;
    let row_height = 30;
    let font_size = 25;
    let col_widths = vec![350, 80, 80, 80, 220, 220];
    let col_acc = col_widths.iter().fold(vec![0], |mut v, e| { v.push(v[v.len() - 1] + e); v });
    let chart_width = col_acc[col_acc.len() - 1];
    let chart_height = row_height * (map_num as i64 + 1);
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
    for (i, MapData { code, name }) in map_iter.enumerate() {
        match save_data.map_stats.get(&code) {
            None => {
               let elems = vec![format!("{}-{}", name.get_name(), sides[code.side]), "-".to_string(), "-".to_string(), "-".to_string(), "-".to_string(), "-".to_string()];
               for (j, text) in elems.into_iter().enumerate() {
                   if j == 0 {
                       chart = chart.draw(centered_text_box(&format!("{}-{}", name.get_name(), sides[code.side])).text_anchor(text_anchor::TextAnchorValue::Start), col_acc[j], row_height / 2 + row_height * (i as i64 + 1));
                   }
                   else {
                       chart = chart.draw(centered_text_box(&text), col_acc[j] + col_widths[j] / 2, row_height / 2 + row_height * (i as i64 + 1));
                   }
               }
            }
            Some(stats) => {
                let ch_text = centered_text_box(&format!("{}-{}", name.get_name(), sides[code.side])).text_anchor(text_anchor::TextAnchorValue::Start);

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
                let row_start = row_height * (i as i64 + 1);
                let row_center = row_height * (i as i64 + 1) + row_height / 2;
                chart = chart
                    .draw(ch_text, col_acc[0], row_center)
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

    for i in 0..map_num {
        let color = color255(222.0, 226.0, 230.0);
        let path = Path::new()
            .line_rel(chart_width, 0)
            .stroke_width(1)
            .stroke(color);
        chart = chart.draw(path, 0, row_height * (i as i64 + 1))
    }
    (chart, chart_width, chart_height)
}

pub fn generate_svg_str<MI>(save_data: &SaveData, map_iter: MI) -> String 
    where MI: IntoIterator<Item=MapData>,
          MI::IntoIter: ExactSizeIterator,
{
    let (chart, _, _) = generate_svg_chart(save_data, map_iter);
    chart.to_string()
}

pub fn generate_png<P, MI>(save_data: &SaveData, map_iter: MI, path: P) -> Result<(), String>
    where MI: IntoIterator<Item=MapData>,
          MI::IntoIter: ExactSizeIterator,
          P: AsRef<std::path::Path>,
{
    let (chart, width, height) = generate_svg_chart(save_data, map_iter);
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
