use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    ActiveTheme,
    highlighter::Language,
    input::{Input, InputEvent, InputState, TabSize},
    resizable::{h_resizable, resizable_panel},
};
use gpui_component_assets::Assets;
use gpui_component_story::Open;
use gpug::{generate_nodes, generate_watts_strogatz_graph, Graph};

pub struct Example {
    input_state: Entity<InputState>,
    graph: Entity<Graph>,
    _subscriptions: Vec<Subscription>,
}

const EXAMPLE: &str = include_str!("../tests/model/vehicle.kdl");

impl Example {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor(Language::Kdl)
                .line_number(true)
                .tab_size(TabSize {
                    tab_size: 2,
                    ..Default::default()
                })
                .searchable(true)
                .placeholder("Enter your KDL diagram here...")
                .default_value(EXAMPLE)
        });

        let graph = cx.new(|cx| {
            let node_count = 15;
            let initial_k = 3;
            let initial_beta = 0.05;
            let nodes = generate_nodes(node_count);
            let edges = generate_watts_strogatz_graph(node_count, initial_k, initial_beta);
            Graph::new(cx, nodes, edges, initial_k, initial_beta)
        });

        let _subscriptions = vec![cx.subscribe(&input_state, |_, _, _: &InputEvent, _| {})];

        Self {
            input_state,
            graph,
            _subscriptions,
        }
    }

    fn on_action_open(&mut self, _: &Open, window: &mut Window, cx: &mut Context<Self>) {
        let path = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: true,
            multiple: false,
            prompt: Some("Select a KDL file".into()),
        });

        let input_state = self.input_state.clone();
        cx.spawn_in(window, async move |_, window| {
            let path = path.await.ok()?.ok()??.iter().next()?.clone();

            let content = std::fs::read_to_string(&path).ok()?;

            window
                .update(|window, cx| {
                    _ = input_state.update(cx, |this, cx| {
                        this.set_value(content, window, cx);
                    });
                })
                .ok();

            Some(())
        })
        .detach();
    }

    fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Render for Example {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("editor")
            .size_full()
            .on_action(cx.listener(Self::on_action_open))
            .child(
                h_resizable("container")
                    .child(
                        resizable_panel().child(
                            div()
                                .id("source")
                                .size_full()
                                .font_family(cx.theme().mono_font_family.clone())
                                .text_size(cx.theme().mono_font_size)
                                .child(
                                    Input::new(&self.input_state)
                                        .h_full()
                                        .p_0()
                                        .border_0()
                                        .focus_bordered(false),
                                ),
                        ),
                    )
                    .child(
                        resizable_panel().child(
                            div()
                                .id("graph-preview")
                                .relative()
                                .w_full()
                                .h_full()
                                .overflow_hidden()
                                .child(self.graph.clone()),
                        ),
                    ),
            )
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        gpui_component_story::init(cx);
        cx.activate(true);

        gpui_component_story::create_new_window("KDL Model Editor", Example::view, cx);
    });
}
