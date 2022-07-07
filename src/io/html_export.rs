use horrorshow::helper::doctype;
use horrorshow::html;
use horrorshow::prelude::*;
use svg::Document;
use svg::node::element::{Group, Rectangle};
use svg::node::element::path::Data;
use svg::node::element::tag::Rectangle;
use svg::node::Text;

use crate::io::json_format::{JsonCP, JsonCPNode, JsonCPNodeType, JsonOrientation, JsonSolution};

#[macro_use]
pub fn generate_solution(json_solution: &JsonSolution) -> String {
    let html = format!("{}", html! {
        : doctype::HTML;
        html(style="font-family:Arial") {
            head {
                title : &json_solution.name;
            }
            body {
                h1 {
                    : &json_solution.name;
                }
                h2 {
                    : format!{"{}", "Statistics"}
                }
                h2 {
                    : format!{"{}", "Cutting Patterns"}
                }
                @ for i in 0..json_solution.cutting_patterns.len() {
                    h3 {
                        : format!("Pattern {}: Object {} [{}x{}], ({:.3}%)",
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
    });

    html
}

pub fn generate_cutting_pattern(json_cp: &JsonCP) -> String {
    let stroke_width = 0.002 * u64::max(json_cp.root.height, json_cp.root.length) as f64;
    let font_size = 0.001 * u64::max(json_cp.root.height, json_cp.root.length) as f64;
    let mut document = Document::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("viewBox", (-stroke_width, -stroke_width, json_cp.root.length as f64 + stroke_width * 2.0, json_cp.root.height as f64 + stroke_width * 2.0));
    let mut group = Group::new();

    let mut subgroups = Vec::new();

    generate_node(&json_cp.root, (0, 0), &mut subgroups, stroke_width, font_size);
    for rect in subgroups {
        group = group.add(rect);
    }
    document = document.add(group);

    let mut write_buffer = Vec::new();
    {
        svg::write(&mut write_buffer, &document).expect("Failed to write SVG");
    }

    std::str::from_utf8(&write_buffer).expect("Failed to convert to string").to_string()
}

fn generate_node(json_cp_node: &JsonCPNode, reference: (u64, u64), groups: &mut Vec<Group>, stroke_width: f64, font_size: f64) {
    match json_cp_node.children.is_empty() {
        true => {
            let color = match json_cp_node.node_type {
                JsonCPNodeType::Structure => panic!("Structure node should have children"),
                JsonCPNodeType::Item => "#BFBFBF",
                JsonCPNodeType::Leftover => "#A9D18E",
            };
            let (x,y) = (reference.0 as f64, reference.1 as f64);
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
                    let mut text = svg::node::element::Text::new()
                        .set("x", x + (width * 0.5))
                        .set("y", y + (height * 0.5))
                        .set("text-anchor", "middle")
                        .set("dominant-baseline", "middle")
                        .set("font-size", format!("{}em",font_size))
                        .set("fill", "black");
                    text = text.add(
                        Text::new(format!("{}: [{}x{}]",
                                          json_cp_node.item.unwrap(),
                                          json_cp_node.length,
                                          json_cp_node.height)
                        )
                    );
                    if json_cp_node.height > json_cp_node.length {
                        text = text.set("transform", format!("rotate(-90 {} {})", x + (width * 0.5), y + (height * 0.5)));
                    }
                    group = group.add(text);
                }
                _ => {}
            };
            groups.push(group);
        }
        false => {
            let mut reference = reference;
            for child in &json_cp_node.children {
                generate_node(child, reference, groups, stroke_width, font_size);
                match json_cp_node.orientation {
                    Some(JsonOrientation::H) => {
                        reference.1 += child.height;
                    }
                    Some(JsonOrientation::V) => {
                        reference.0 += child.length;
                    }
                    _ => { panic!("Node with children should have orientation") }
                }
            }
        }
    }
}