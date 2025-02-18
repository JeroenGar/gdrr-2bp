use horrorshow::helper::doctype;
use horrorshow::html;
use horrorshow::prelude::*;
use svg::node::element::{Group, Rectangle, Text};
use svg::Document;

use crate::io::json_format::{JsonCP, JsonCPNode, JsonCPNodeType, JsonOrientation, JsonSolution};

pub fn generate_solution(json_solution: &JsonSolution) -> String {
    let html = format!(
        "{}",
        html! {
            : doctype::HTML;
            html(style="font-family:Arial") {
                head {
                    title : format!("Solution {}", &json_solution.name);
                }
                body {
                    h1 {
                        : format!("Solution {}", &json_solution.name);
                    }
                    h2 {
                        : format!{"{}", "Statistics"}
                    }
                    table {
                        tr {
                            th(style="text-align:left") {
                                : "Usage";
                            }
                            td {
                                : format!{"{:.3}%", json_solution.statistics.usage_pct};
                            }
                        }
                        tr {
                            th(style="text-align:left") {
                                : "Part area included";
                            }
                            td {
                                : format!{"{:.3}%", json_solution.statistics.part_area_included_pct};
                            }
                        }
                        tr {
                            th(style="text-align:left") {
                                : "# Objects used";
                            }
                            td {
                                : format!{"{}", json_solution.statistics.n_objects_used};
                            }
                        }
                        tr {
                            th(style="text-align:left") {
                                : "Material cost";
                            }
                            td {
                                : format!{"{}", json_solution.statistics.material_cost};
                            }
                        }
                        tr {
                            th(style="text-align:left") {
                                : "Run time";
                            }
                            td {
                                : format!{"{}s", json_solution.statistics.run_time_ms as f64 / 1000.0};
                            }
                        }
                        tr {
                            th(style="text-align:left") {
                                : "Config path";
                            }
                            td {
                                : format!{"{}",  json_solution.statistics.config_path};
                            }
                        }
                    }
                    h2 {
                        : format!{"{}", "Cutting Patterns"}
                    }
                    @ for i in 0..json_solution.cutting_patterns.len() {
                        h3 {
                            : format!("Pattern {}: Object {} [{}x{}], {:.3}% usage",
                                i,
                                json_solution.cutting_patterns[i].object,
                                json_solution.cutting_patterns[i].root.length,
                                json_solution.cutting_patterns[i].root.height,
                                json_solution.cutting_patterns[i].usage * 100.0
                            );
                        }
                        div(style="width:1000px;") {
                            : Raw(generate_cutting_pattern(&json_solution.cutting_patterns[i]))
                        }

                    }
                }
            }
        }
    );

    html
}

pub fn generate_cutting_pattern(json_cp: &JsonCP) -> String {
    let stroke_width = 0.002 * u64::max(json_cp.root.height, json_cp.root.length) as f64;
    let mut document = Document::new()
        .set("width", "100%")
        .set("height", "100%")
        .set(
            "viewBox",
            (
                -stroke_width,
                -stroke_width,
                json_cp.root.length as f64 + stroke_width * 2.0,
                json_cp.root.height as f64 + stroke_width * 2.0,
            ),
        );
    let mut group = Group::new();

    let mut subgroups = Vec::new();

    generate_node(&json_cp.root, (0, 0), &mut subgroups, stroke_width);
    for rect in subgroups {
        group = group.add(rect);
    }
    document = document.add(group);

    let mut write_buffer = Vec::new();
    {
        svg::write(&mut write_buffer, &document).expect("Failed to write SVG");
    }

    std::str::from_utf8(&write_buffer)
        .expect("Failed to convert to string")
        .to_string()
}

fn generate_node(
    json_cp_node: &JsonCPNode,
    reference: (u64, u64),
    groups: &mut Vec<Group>,
    stroke_width: f64,
) {
    match json_cp_node.children.is_empty() {
        true => {
            let color = match json_cp_node.node_type {
                JsonCPNodeType::Structure => panic!("Structure node should have children"),
                JsonCPNodeType::Item => "#BFBFBF",
                JsonCPNodeType::Leftover => "#A9D18E",
            };
            let (x, y) = (reference.0 as f64, reference.1 as f64);
            let (width, height) = (json_cp_node.length as f64, json_cp_node.height as f64);
            let mut group = Group::new();
            let rect = Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", width)
                .set("height", height)
                .set("fill", color)
                .set("stroke", "black")
                .set("stroke-width", stroke_width.to_string());
            group = group.add(rect);

            match json_cp_node.node_type {
                JsonCPNodeType::Item => {
                    let mut text = Text::new(format!(
                        "{}: [{}x{}]",
                        json_cp_node.item.unwrap(),
                        json_cp_node.length,
                        json_cp_node.height
                    ))
                    .set("x", x + (width * 0.5))
                    .set("y", y + (height * 0.5))
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "middle")
                    .set("fill", "black");

                    if json_cp_node.height > json_cp_node.length {
                        text = text.set(
                            "transform",
                            format!("rotate(-90 {} {})", x + (width * 0.5), y + (height * 0.5)),
                        );
                    }
                    let font_size = f64::min(
                        0.005 * u64::max(json_cp_node.height, json_cp_node.length) as f64,
                        0.02 * u64::min(json_cp_node.height, json_cp_node.length) as f64,
                    );
                    text = text.set("font-size", format!("{}em", font_size));

                    group = group.add(text);
                }
                _ => {}
            };
            groups.push(group);
        }
        false => {
            let mut reference = reference;
            for child in &json_cp_node.children {
                generate_node(child, reference, groups, stroke_width);
                match json_cp_node.orientation {
                    Some(JsonOrientation::H) => {
                        reference.1 += child.height;
                    }
                    Some(JsonOrientation::V) => {
                        reference.0 += child.length;
                    }
                    _ => {
                        panic!("Node with children should have orientation")
                    }
                }
            }
        }
    }
}
