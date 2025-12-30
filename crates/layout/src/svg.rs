use crate::types::*;
use std::fs;

pub fn generate_svg(result: &LayoutResult, filename: &str, show_grid: bool, show_all_ports: bool) {
    use std::collections::HashSet;
    let mut used_ports = HashSet::new();

    for edge in &result.edges {
        let source_node = &result.nodes[edge.source];
        let target_node = &result.nodes[edge.target];

        // Determine side for source
        let dx = target_node.position.x + target_node.size.width / 2.0
            - (source_node.position.x + source_node.size.width / 2.0);
        let dy = target_node.position.y + target_node.size.height / 2.0
            - (source_node.position.y + source_node.size.height / 2.0);
        let side = if dx.abs() > dy.abs() {
            if dx > 0.0 { "right" } else { "left" }
        } else if dy > 0.0 {
            "bottom"
        } else {
            "top"
        };
        let port_index = match side {
            "left" => 0,
            "right" => 1,
            "top" => 2,
            "bottom" => 3,
            _ => 0,
        };
        used_ports.insert((edge.source, port_index));

        // Determine side for target
        let dx = source_node.position.x + source_node.size.width / 2.0
            - (target_node.position.x + target_node.size.width / 2.0);
        let dy = source_node.position.y + source_node.size.height / 2.0
            - (target_node.position.y + target_node.size.height / 2.0);
        let side = if dx.abs() > dy.abs() {
            if dx > 0.0 { "right" } else { "left" }
        } else if dy > 0.0 {
            "bottom"
        } else {
            "top"
        };
        let port_index = match side {
            "left" => 0,
            "right" => 1,
            "top" => 2,
            "bottom" => 3,
            _ => 0,
        };
        used_ports.insert((edge.target, port_index));
    }

    let mut svg = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
<rect width="100%" height="100%" fill="white"/>
"#,
        result.canvas_width, result.canvas_height
    );

    // Add grid lines if requested
    if show_grid {
        let cell_size = 5.0;
        let num_x_lines = (result.canvas_width / cell_size).ceil() as usize;
        let num_y_lines = (result.canvas_height / cell_size).ceil() as usize;

        // Vertical grid lines
        for i in 0..=num_x_lines {
            let x = i as f64 * cell_size;
            svg.push_str(&format!(
                r#"<line x1="{}" y1="0" x2="{}" y2="{}" stroke="lightgray" stroke-width="0.5"/>
"#,
                x, x, result.canvas_height
            ));
        }

        // Horizontal grid lines
        for i in 0..=num_y_lines {
            let y = i as f64 * cell_size;
            svg.push_str(&format!(
                r#"<line x1="0" y1="{}" x2="{}" y2="{}" stroke="lightgray" stroke-width="0.5"/>
"#,
                y, result.canvas_width, y
            ));
        }
    }

    for node in result.nodes.iter() {
        let fill_color = node
            .attributes
            .iter()
            .find(|(k, _)| k == "color")
            .map(|(_, v)| v.as_str())
            .unwrap_or("lightblue");

        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="black"/>
<text x="{}" y="{}" font-family="Arial" font-size="12" text-anchor="middle">{}</text>
"#,
            node.position.x,
            node.position.y,
            node.size.width,
            node.size.height,
            fill_color,
            node.position.x + node.size.width / 2.0,
            node.position.y + node.size.height / 2.0 + 5.0,
            node.id
        ));
    }

    for edge in &result.edges {
        if edge.path.len() >= 2 {
            let mut path_data = format!("M {} {}", edge.path[0].x, edge.path[0].y);
            for point in &edge.path[1..] {
                path_data.push_str(&format!(" L {} {}", point.x, point.y));
            }
            svg.push_str(&format!(
                r#"<path d="{}" stroke="black" stroke-width="2" fill="none"/>
"#,
                path_data
            ));
        }
    }

    for (node_index, node) in result.nodes.iter().enumerate() {
        for (port_index, port) in node.ports.iter().enumerate() {
            if show_all_ports || used_ports.contains(&(node_index, port_index)) {
                let fill = match port.port_type {
                    PortType::Input => "lightblue",
                    PortType::Output => "lightcoral",
                };
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="black"/>
"#,
                    node.position.x + port.position.x,
                    node.position.y + port.position.y,
                    port.size.width,
                    port.size.height,
                    fill
                ));
            }
        }
    }

    svg.push_str("</svg>");

    fs::write(filename, svg).expect("Unable to write SVG");
}
