use archviz_layout::*;

#[test]
fn test_layout_set_1() {
    let nodes = vec![
        Node {
            id: "ECU1".to_string(),
            size: Size {
                width: 120.0,
                height: 80.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 35.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 120.0, y: 35.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 55.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 55.0, y: 80.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "ECU2".to_string(),
            size: Size {
                width: 100.0,
                height: 60.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 25.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 100.0, y: 25.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 45.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 45.0, y: 60.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Sensor".to_string(),
            size: Size {
                width: 80.0,
                height: 40.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 15.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 80.0, y: 15.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 35.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 35.0, y: 40.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
    ];
    let edges = vec![(0, 1), (1, 2), (0, 2)];

    let layout = CustomLayout {
        min_spacing: 120.0,
        ..Default::default()
    };
    let result = layout.layout(nodes, edges);

    println!("Set 1 Nodes:");
    for node in &result.nodes {
        println!(
            "  {}: size {:?}, position {:?}",
            node.id, node.size, node.position
        );
    }
    println!("Set 1 Edges:");
    for edge in &result.edges {
        println!("  {} -> {}: {:?}", edge.source, edge.target, edge.path);
    }

    generate_svg(&result, "test_set_1.svg", true, true);
}

#[test]
fn test_layout_set_2() {
    let nodes = vec![
        Node {
            id: "Gateway".to_string(),
            size: Size {
                width: 150.0,
                height: 100.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 45.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 150.0, y: 45.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 70.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 70.0, y: 100.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Display".to_string(),
            size: Size {
                width: 90.0,
                height: 70.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 30.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 90.0, y: 30.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 40.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 40.0, y: 70.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Battery".to_string(),
            size: Size {
                width: 60.0,
                height: 50.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 60.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 25.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 25.0, y: 50.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Motor".to_string(),
            size: Size {
                width: 110.0,
                height: 90.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 40.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 110.0, y: 40.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 50.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 50.0, y: 90.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
    ];
    let edges = vec![(0, 1), (0, 2), (0, 3), (1, 3), (2, 3)];

    let layout = CustomLayout {
        spaced_edges: true,
        min_spacing: 120.0,
        ..Default::default()
    };
    let result = layout.layout(nodes, edges);

    println!("Set 2 Nodes:");
    for node in &result.nodes {
        println!(
            "  {}: size {:?}, position {:?}",
            node.id, node.size, node.position
        );
    }
    println!("Set 2 Edges:");
    for edge in &result.edges {
        println!("  {} -> {}: {:?}", edge.source, edge.target, edge.path);
    }

    generate_svg(&result, "test_set_2.svg", false, false);
}

#[test]
fn test_layout_set_3() {
    let nodes = vec![
        Node {
            id: "ABS".to_string(),
            size: Size {
                width: 100.0,
                height: 60.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 25.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 100.0, y: 25.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 45.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 45.0, y: 60.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "ESP".to_string(),
            size: Size {
                width: 120.0,
                height: 80.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 35.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 120.0, y: 35.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 55.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 55.0, y: 80.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Airbag".to_string(),
            size: Size {
                width: 90.0,
                height: 50.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 90.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 40.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 40.0, y: 50.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Climate".to_string(),
            size: Size {
                width: 110.0,
                height: 70.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 30.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 110.0, y: 30.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 50.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 50.0, y: 70.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Infotainment".to_string(),
            size: Size {
                width: 140.0,
                height: 100.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 45.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 140.0, y: 45.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 65.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 65.0, y: 100.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
    ];
    let edges = vec![(0, 1), (1, 2), (2, 3), (3, 4), (0, 4), (1, 3)];

    let layout = CustomLayout {
        allow_diagonals: false,
        min_spacing: 120.0,
        ..Default::default()
    };
    let result = layout.layout(nodes, edges);

    println!("Set 3 Nodes:");
    for node in &result.nodes {
        println!(
            "  {}: size {:?}, position {:?}",
            node.id, node.size, node.position
        );
    }
    println!("Set 3 Edges:");
    for edge in &result.edges {
        println!("  {} -> {}: {:?}", edge.source, edge.target, edge.path);
    }

    generate_svg(&result, "test_set_3.svg", true, false);
}

#[test]
fn test_layout_set_4() {
    let nodes = vec![
        Node {
            id: "Engine".to_string(),
            size: Size {
                width: 140.0,
                height: 90.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 40.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 140.0, y: 40.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 65.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 65.0, y: 90.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Transmission".to_string(),
            size: Size {
                width: 120.0,
                height: 70.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 30.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 120.0, y: 30.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 55.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 55.0, y: 70.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Brakes".to_string(),
            size: Size {
                width: 100.0,
                height: 60.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 25.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 100.0, y: 25.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 45.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 45.0, y: 60.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Steering".to_string(),
            size: Size {
                width: 110.0,
                height: 65.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 27.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 110.0, y: 27.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 50.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 50.0, y: 65.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Sensors".to_string(),
            size: Size {
                width: 90.0,
                height: 50.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 90.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 40.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 40.0, y: 50.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Dashboard".to_string(),
            size: Size {
                width: 130.0,
                height: 75.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 32.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 130.0, y: 32.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 60.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 60.0, y: 75.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "ECU".to_string(),
            size: Size {
                width: 100.0,
                height: 55.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 22.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 100.0, y: 22.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 45.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 45.0, y: 55.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "FuelPump".to_string(),
            size: Size {
                width: 85.0,
                height: 45.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 17.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 85.0, y: 17.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 37.5, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 37.5, y: 45.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Alternator".to_string(),
            size: Size {
                width: 95.0,
                height: 55.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 22.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 95.0, y: 22.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 42.5, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 42.5, y: 55.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Radiator".to_string(),
            size: Size {
                width: 110.0,
                height: 70.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 30.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 110.0, y: 30.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 50.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 50.0, y: 70.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Battery".to_string(),
            size: Size {
                width: 80.0,
                height: 60.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 25.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 80.0, y: 25.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 35.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 35.0, y: 60.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "AirFilter".to_string(),
            size: Size {
                width: 75.0,
                height: 40.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 15.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 75.0, y: 15.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 32.5, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 32.5, y: 40.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Exhaust".to_string(),
            size: Size {
                width: 125.0,
                height: 50.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 125.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 57.5, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 57.5, y: 50.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        Node {
            id: "Catalytic".to_string(),
            size: Size {
                width: 105.0,
                height: 45.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 17.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 105.0, y: 17.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 47.5, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 47.5, y: 45.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
    ];
    // Complex connectivity with cross-connections and potential routing challenges
    let edges = vec![
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 5),
        (5, 6), // main chain
        (0, 3),
        (1, 4),
        (2, 5),
        (0, 6), // cross connections
        (3, 6),
        (4, 6), // additional connections to ECU
        // New component connections
        (0, 7),
        (7, 8),
        (8, 9),
        (9, 10), // fuel system chain
        (1, 11),
        (11, 12),
        (12, 13), // electrical system chain
        (2, 9),
        (3, 10),
        (4, 11), // cooling system connections
        (5, 12),
        (6, 13), // exhaust system connections
        (7, 10),
        (8, 13),
        (9, 6), // additional cross-connections
    ];

    let layout = CustomLayout {
        allow_diagonals: false,
        min_spacing: 120.0,
        ..Default::default()
    };
    let result = layout.layout(nodes, edges);

    println!("Set 4 Nodes:");
    for node in &result.nodes {
        println!(
            "  {}: size {:?}, position {:?}",
            node.id, node.size, node.position
        );
    }
    println!("Set 4 Edges:");
    for edge in &result.edges {
        println!("  {} -> {}: {:?}", edge.source, edge.target, edge.path);
    }

    generate_svg(&result, "test_set_4.svg", true, false);
}

#[test]
fn test_layout_set_5() {
    let nodes = vec![
        // CAN Bus node with 8 ports
        Node {
            id: "CAN_Bus".to_string(),
            size: Size {
                width: 200.0,
                height: 60.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                // Left side inputs (2 ports)
                Port {
                    position: Position { x: -10.0, y: 15.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: -10.0, y: 35.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                // Right side outputs (2 ports)
                Port {
                    position: Position { x: 200.0, y: 15.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 200.0, y: 35.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                // Top side inputs (2 ports)
                Port {
                    position: Position { x: 45.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 145.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                // Bottom side outputs (2 ports)
                Port {
                    position: Position { x: 45.0, y: 60.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 145.0, y: 60.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![("color".to_string(), "grey".to_string())],
        },
        // Ethernet Backbone node with 4 ports
        Node {
            id: "Ethernet_Backbone".to_string(),
            size: Size {
                width: 180.0,
                height: 50.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                // Left side input
                Port {
                    position: Position { x: -10.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                // Right side output
                Port {
                    position: Position { x: 180.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                // Top side input
                Port {
                    position: Position { x: 85.0, y: -10.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                // Bottom side output
                Port {
                    position: Position { x: 85.0, y: 50.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![("color".to_string(), "grey".to_string())],
        },
        // Gateway ECU connected to both CAN and Ethernet
        Node {
            id: "Gateway_ECU".to_string(),
            size: Size {
                width: 120.0,
                height: 80.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: -10.0, y: 50.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 120.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
                Port {
                    position: Position { x: 120.0, y: 50.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![("color".to_string(), "violet".to_string())],
        },
        // Engine ECU connected to CAN
        Node {
            id: "Engine_ECU".to_string(),
            size: Size {
                width: 100.0,
                height: 60.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 25.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 100.0, y: 25.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        // Transmission ECU connected to CAN
        Node {
            id: "Transmission_ECU".to_string(),
            size: Size {
                width: 130.0,
                height: 70.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 30.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 130.0, y: 30.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        // Body Control ECU connected to CAN
        Node {
            id: "Body_ECU".to_string(),
            size: Size {
                width: 110.0,
                height: 65.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 27.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 110.0, y: 27.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        // Telematics ECU connected to Ethernet
        Node {
            id: "Telematics_ECU".to_string(),
            size: Size {
                width: 120.0,
                height: 55.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 22.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 120.0, y: 22.5 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
        // ADAS ECU connected to Ethernet
        Node {
            id: "ADAS_ECU".to_string(),
            size: Size {
                width: 100.0,
                height: 50.0,
            },
            position: Position { x: 0.0, y: 0.0 },
            ports: vec![
                Port {
                    position: Position { x: -10.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Input,
                },
                Port {
                    position: Position { x: 100.0, y: 20.0 },
                    size: Size {
                        width: 10.0,
                        height: 10.0,
                    },
                    port_type: PortType::Output,
                },
            ],
            attributes: vec![],
        },
    ];

    // Less interconnected than set 4 - focused on bus topology
    let edges = vec![
        // Gateway ECU connections to CAN Bus (2 connections)
        (0, 2),
        (2, 0),
        // Gateway ECU connections to Ethernet Backbone (2 connections)
        (1, 2),
        (2, 1),
        // CAN-connected ECUs (3 ECUs connected to CAN Bus)
        (0, 3),
        (3, 0), // Engine ECU
        (0, 4),
        (4, 0), // Transmission ECU
        (0, 5),
        (5, 0), // Body ECU
        // Ethernet-connected ECUs (2 ECUs connected to Ethernet Backbone)
        (1, 6),
        (6, 1), // Telematics ECU
        (1, 7),
        (7, 1), // ADAS ECU
    ];

    let layout = CustomLayout {
        allow_diagonals: false,
        min_spacing: 120.0,
        ..Default::default()
    };
    let result = layout.layout(nodes, edges);

    println!("Set 5 Nodes:");
    for node in &result.nodes {
        println!(
            "  {}: size {:?}, position {:?}",
            node.id, node.size, node.position
        );
    }
    println!("Set 5 Edges:");
    for edge in &result.edges {
        println!("  {} -> {}: {:?}", edge.source, edge.target, edge.path);
    }

    generate_svg(&result, "test_set_5.svg", true, true);
}

#[test]
fn test_layout_in_place() {
    // Create test data structures that implement the traits
    #[derive(Debug, Clone)]
    struct TestNode {
        id: String,
        position: Position,
        size: Size,
        ports: Vec<Port>,
    }

    impl LayoutNode for TestNode {
        fn position(&self) -> Position {
            self.position.clone()
        }
        fn size(&self) -> Size {
            self.size.clone()
        }
        fn set_position(&mut self, pos: Position) {
            self.position = pos;
        }
        fn id(&self) -> String {
            self.id.clone()
        }
        fn ports(&self) -> Vec<Port> {
            self.ports.clone()
        }
    }

    #[derive(Debug, Clone)]
    struct TestEdge {
        source: usize,
        target: usize,
        path: Vec<Position>,
    }

    impl LayoutEdge for TestEdge {
        fn source(&self) -> usize {
            self.source
        }
        fn target(&self) -> usize {
            self.target
        }
        fn set_path(&mut self, path: Vec<Position>) {
            self.path = path;
        }
    }

    // Create test nodes and edges with closer initial positions
    let mut nodes = vec![
        TestNode {
            id: "A".to_string(),
            position: Position { x: 0.0, y: 0.0 },
            size: Size {
                width: 100.0,
                height: 50.0,
            },
            ports: vec![],
        },
        TestNode {
            id: "B".to_string(),
            position: Position { x: 50.0, y: 0.0 }, // Closer together
            size: Size {
                width: 100.0,
                height: 50.0,
            },
            ports: vec![],
        },
    ];

    let mut edges = vec![TestEdge {
        source: 0,
        target: 1,
        path: vec![],
    }];

    // Create layout configuration with more iterations
    let config = CustomLayout {
        iterations: 100,
        repulsion_strength: 10000.0,
        attraction_strength: 0.01,
        min_spacing: 50.0,
        allow_diagonals: true,
        spaced_edges: true,
    };

    // Run in-place layout
    let result = layout_in_place(&mut nodes, &mut edges, &config);

    // Verify the result
    assert!(result.is_ok());

    // Check that positions have been updated (should move apart due to repulsion)
    assert_ne!(nodes[0].position.x, 0.0);
    assert_ne!(nodes[1].position.x, 50.0); // Should move away from each other

    // Check that edge path has been set
    assert!(!edges[0].path.is_empty());
}
