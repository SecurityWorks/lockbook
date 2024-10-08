use pulldown_cmark::LinkType;

use crate::tab::{markdown_editor, ExtendedOutput};
use crate::tab::{
    markdown_editor::{
        input::{Event, Region},
        style::{BlockNode, InlineNode, ListItem, MarkdownNode},
        Editor,
    },
    ExtendedInput as _,
};

use crate::theme::icons::Icon;

use super::Button;

pub const MOBILE_TOOL_BAR_SIZE: f32 = 45.0;

#[derive(Clone)]
pub struct ToolbarButton {
    icon: Icon,
    id: String,
    callback: fn(&mut egui::Ui, &mut ToolBar, &mut markdown_editor::Response),
}

#[derive(Clone)]
pub enum Component {
    Button(ToolbarButton),
    Separator(egui::Margin),
}

#[derive(Clone)]
pub struct ToolBar {
    pub margin: egui::Margin,
    id: egui::Id,
    pub has_focus: bool,
    buttons: Vec<ToolbarButton>,
    mobile_components: Vec<Component>,
    hide_keyboard_components: Vec<Component>,
    header_click_count: usize,
    pub visibility: ToolBarVisibility,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ToolBarVisibility {
    Minimized,
    Maximized,
    Disabled,
}

impl ToolBar {
    pub fn new(visibility: &ToolBarVisibility) -> Self {
        Self {
            margin: if cfg!(target_os = "ios") || cfg!(target_os = "android") {
                egui::Margin { left: 10.0, right: 23.0, top: 5.0, bottom: 5.0 }
            } else {
                egui::Margin::symmetric(15.0, 0.0)
            },
            buttons: get_buttons(visibility),
            mobile_components: get_mobile_components(),
            hide_keyboard_components: get_hide_keyboard_components(),
            header_click_count: 1,
            has_focus: false,
            visibility: visibility.to_owned(),
            id: egui::Id::NULL,
        }
    }

    pub fn show(
        &mut self, ui: &mut egui::Ui, editor: &mut Editor, res: &mut markdown_editor::Response,
    ) {
        if cfg!(target_os = "ios") || cfg!(target_os = "android") {
            ui.allocate_ui(egui::vec2(ui.available_width(), MOBILE_TOOL_BAR_SIZE), |ui| {
                egui::Frame::default()
                    .inner_margin(self.margin)
                    .show(ui, |ui| self.map_buttons(ui, editor, res, true));
            });
        } else {
            let toolbar_rect = self.calculate_rect(ui, editor);
            if ui.available_rect_before_wrap().width() < toolbar_rect.width() {
                return;
            }

            self.id = ui.id();

            ui.allocate_ui_at_rect(toolbar_rect, |ui| {
                egui::Frame::default()
                    .fill(ui.visuals().code_bg_color)
                    .inner_margin(self.margin)
                    .shadow(egui::epaint::Shadow {
                        offset: ui.visuals().window_shadow.offset,
                        blur: ui.visuals().window_shadow.blur,
                        spread: ui.visuals().window_shadow.spread,
                        color: ui.visuals().window_shadow.color.gamma_multiply(0.3),
                    })
                    .rounding(ui.style().visuals.menu_rounding)
                    .show(ui, |ui| self.map_buttons(ui, editor, res, false))
            });
        }
    }

    fn map_buttons(
        &mut self, ui: &mut egui::Ui, editor: &mut Editor,
        editor_res: &mut markdown_editor::Response, is_mobile: bool,
    ) {
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().button_padding =
                    if is_mobile { egui::vec2(0.0, 5.0) } else { egui::vec2(10.0, 20.0) };

                if is_mobile {
                    if editor.is_virtual_keyboard_showing {
                        self.process_mobile_components(
                            self.hide_keyboard_components.clone(),
                            ui,
                            editor_res,
                        );
                    }

                    self.process_mobile_components(self.mobile_components.clone(), ui, editor_res);
                } else {
                    self.buttons.clone().iter().for_each(|btn| {
                        let res = Button::default().icon(&btn.icon).show(ui);
                        if res.hovered() && btn.id != "header" {
                            self.header_click_count = 1;
                        }

                        if res.clicked() {
                            (btn.callback)(ui, self, editor_res);
                            ui.ctx().request_repaint();
                        }
                        if btn.id == "header" {
                            res.on_hover_text(format!("H{}", self.header_click_count));
                        }
                    });
                };
            })
        });
    }

    fn process_mobile_components(
        &mut self, components: Vec<Component>, ui: &mut egui::Ui,
        editor_res: &mut markdown_editor::Response,
    ) {
        components.iter().for_each(|comp| match comp {
            Component::Button(btn) => {
                ui.add_space(10.0);
                let res = Button::default().icon(&btn.icon).show(ui);
                ui.add_space(10.0);

                if res.clicked() {
                    (btn.callback)(ui, self, editor_res);
                }
            }
            Component::Separator(sep) => {
                ui.add_space(sep.right);
                ui.add(egui::Separator::default().shrink(ui.available_height() * 0.3));
                ui.add_space(sep.left);
            }
        });
    }

    fn width(&self) -> f32 {
        let icon_width = 44; // icon default width is 34 and spacing is defined below as 10
        let width = icon_width * self.buttons.len() + self.margin.sum().x as usize;
        width as f32
    }

    /// center the toolbar relative to the editor
    fn calculate_rect(&self, ui: &mut egui::Ui, editor: &mut Editor) -> egui::Rect {
        let on = match self.visibility {
            ToolBarVisibility::Minimized | ToolBarVisibility::Disabled => true,
            ToolBarVisibility::Maximized => false,
        };
        let how_on = ui.ctx().animate_bool(egui::Id::from("toolbar_animate"), on);

        let editor_rect = editor.rect;

        let maximized_min_x = (editor_rect.width() - self.width()) / 2.0 + editor_rect.left();
        let minimized_min_x = editor_rect.max.x - (self.width() / self.buttons.len() as f32) - 40.0;

        let min_pos = egui::Pos2 {
            x: egui::lerp((maximized_min_x)..=(minimized_min_x), how_on),
            y: editor_rect.bottom() - 90.0,
        };

        let maximized_max_x = editor_rect.right() - (editor_rect.width() - self.width()) / 2.0;
        let minimized_max_x = editor_rect.right();

        let max_pos = egui::Pos2 {
            x: egui::lerp((maximized_max_x)..=(minimized_max_x), how_on),
            y: editor_rect.bottom(),
        };

        match self.visibility {
            ToolBarVisibility::Maximized | ToolBarVisibility::Minimized => {
                egui::Rect { min: min_pos, max: max_pos }
            }
            ToolBarVisibility::Disabled => egui::Rect::NOTHING,
        }
    }
}

fn get_buttons(visibility: &ToolBarVisibility) -> Vec<ToolbarButton> {
    match visibility {
        ToolBarVisibility::Minimized => {
            vec![ToolbarButton {
                icon: Icon::VISIBILITY_ON,
                id: "visibility_on".to_string(),
                callback: |_, t, _| {
                    t.visibility = ToolBarVisibility::Maximized;
                    t.buttons = get_buttons(&t.visibility);
                },
            }]
        }
        ToolBarVisibility::Maximized => {
            vec![
                ToolbarButton {
                    icon: Icon::HEADER_1,
                    id: "header".to_string(),
                    callback: toggle_heading_style,
                },
                ToolbarButton {
                    icon: Icon::BOLD,
                    id: "bold".to_string(),
                    callback: |ui, _, _| {
                        ui.ctx().push_markdown_event(Event::ToggleStyle {
                            region: Region::Selection,
                            style: MarkdownNode::Inline(InlineNode::Bold),
                        })
                    },
                },
                ToolbarButton {
                    icon: Icon::ITALIC,
                    id: "italic".to_string(),
                    callback: |ui, _, _| {
                        ui.ctx().push_markdown_event(Event::ToggleStyle {
                            region: Region::Selection,
                            style: MarkdownNode::Inline(InlineNode::Italic),
                        })
                    },
                },
                ToolbarButton {
                    icon: Icon::CODE,
                    id: "in_line_code".to_string(),
                    callback: |ui, _, _| {
                        ui.ctx().push_markdown_event(Event::ToggleStyle {
                            region: Region::Selection,
                            style: MarkdownNode::Inline(InlineNode::Code),
                        });
                    },
                },
                ToolbarButton {
                    icon: Icon::STRIKETHROUGH,
                    id: "strikethrough".to_string(),
                    callback: |ui, _, _| {
                        ui.ctx().push_markdown_event(Event::ToggleStyle {
                            region: Region::Selection,
                            style: MarkdownNode::Inline(InlineNode::Strikethrough),
                        });
                    },
                },
                ToolbarButton {
                    icon: Icon::NUMBER_LIST,
                    id: "number_list".to_string(),
                    callback: |ui, _, _| {
                        ui.ctx().push_markdown_event(Event::toggle_block_style(
                            BlockNode::ListItem(ListItem::Numbered(1), 0),
                        ))
                    },
                },
                ToolbarButton {
                    icon: Icon::BULLET_LIST,
                    id: "bullet_list".to_string(),
                    callback: |ui, _, _| {
                        ui.ctx().push_markdown_event(Event::toggle_block_style(
                            BlockNode::ListItem(ListItem::Bulleted, 0),
                        ))
                    },
                },
                ToolbarButton {
                    icon: Icon::TODO_LIST,
                    id: "todo_list".to_string(),
                    callback: |ui, _, _| {
                        ui.ctx().push_markdown_event(Event::toggle_block_style(
                            BlockNode::ListItem(ListItem::Todo(false), 0),
                        ))
                    },
                },
                ToolbarButton {
                    icon: Icon::LINK,
                    id: "link".to_string(),
                    callback: |ui, _, _| {
                        ui.ctx().push_markdown_event(Event::ToggleStyle {
                            region: Region::Selection,
                            style: MarkdownNode::Inline(InlineNode::Link(
                                LinkType::Inline,
                                "".into(),
                                "".into(),
                            )),
                        });
                    },
                },
                ToolbarButton {
                    icon: Icon::VISIBILITY_OFF,
                    id: "visibility_off".to_string(),
                    callback: |_, t, _| {
                        t.visibility = ToolBarVisibility::Minimized;
                        t.buttons = get_buttons(&t.visibility);
                    },
                },
            ]
        }
        ToolBarVisibility::Disabled => vec![],
    }
}

fn get_hide_keyboard_components() -> Vec<Component> {
    vec![
        Component::Button(ToolbarButton {
            icon: Icon::KEYBOARD_HIDE,
            id: "keyboard_hide".to_string(),
            callback: |ui, _, _| {
                ui.ctx().set_virtual_keyboard_shown(false);
            },
        }),
        Component::Separator(egui::Margin::symmetric(10.0, 0.0)),
    ]
}

fn get_mobile_components() -> Vec<Component> {
    vec![
        Component::Button(ToolbarButton {
            icon: Icon::HEADER_1,
            id: "header".to_string(),
            callback: toggle_heading_style,
        }),
        Component::Button(ToolbarButton {
            icon: Icon::BOLD,
            id: "bold".to_string(),
            callback: |ui, _, _| {
                ui.ctx().push_markdown_event(Event::ToggleStyle {
                    region: Region::Selection,
                    style: MarkdownNode::Inline(InlineNode::Bold),
                })
            },
        }),
        Component::Button(ToolbarButton {
            icon: Icon::ITALIC,
            id: "italic".to_string(),
            callback: |ui, _, _| {
                ui.ctx().push_markdown_event(Event::ToggleStyle {
                    region: Region::Selection,
                    style: MarkdownNode::Inline(InlineNode::Italic),
                })
            },
        }),
        Component::Button(ToolbarButton {
            icon: Icon::CODE,
            id: "in_line_code".to_string(),
            callback: |ui, _, _| {
                ui.ctx().push_markdown_event(Event::ToggleStyle {
                    region: Region::Selection,
                    style: MarkdownNode::Inline(InlineNode::Code),
                });
            },
        }),
        Component::Button(ToolbarButton {
            icon: Icon::STRIKETHROUGH,
            id: "strikethrough".to_string(),
            callback: |ui, _, _| {
                ui.ctx().push_markdown_event(Event::ToggleStyle {
                    region: Region::Selection,
                    style: MarkdownNode::Inline(InlineNode::Strikethrough),
                });
            },
        }),
        Component::Separator(egui::Margin::symmetric(10.0, 0.0)),
        Component::Button(ToolbarButton {
            icon: Icon::NUMBER_LIST,
            id: "number_list".to_string(),
            callback: |ui, _, _| {
                ui.ctx()
                    .push_markdown_event(Event::toggle_block_style(BlockNode::ListItem(
                        ListItem::Numbered(1),
                        0,
                    )))
            },
        }),
        Component::Button(ToolbarButton {
            icon: Icon::BULLET_LIST,
            id: "bullet_list".to_string(),
            callback: |ui, _, _| {
                ui.ctx()
                    .push_markdown_event(Event::toggle_block_style(BlockNode::ListItem(
                        ListItem::Bulleted,
                        0,
                    )))
            },
        }),
        Component::Button(ToolbarButton {
            icon: Icon::TODO_LIST,
            id: "todo_list".to_string(),
            callback: |ui, _, _| {
                ui.ctx()
                    .push_markdown_event(Event::toggle_block_style(BlockNode::ListItem(
                        ListItem::Todo(false),
                        0,
                    )))
            },
        }),
        Component::Separator(egui::Margin::symmetric(10.0, 0.0)),
        Component::Button(ToolbarButton {
            icon: Icon::LINK,
            id: "link".to_string(),
            callback: |ui, _, _| {
                ui.ctx().push_markdown_event(Event::ToggleStyle {
                    region: Region::Selection,
                    style: MarkdownNode::Inline(InlineNode::Link(
                        LinkType::Inline,
                        "".into(),
                        "".into(),
                    )),
                });
            },
        }),
        Component::Separator(egui::Margin::symmetric(10.0, 0.0)),
        Component::Button(ToolbarButton {
            icon: Icon::INDENT,
            id: "indent".to_string(),
            callback: |ui, _, _| {
                ui.ctx()
                    .push_markdown_event(Event::Indent { deindent: false })
            },
        }),
        Component::Button(ToolbarButton {
            icon: Icon::DEINDENT,
            id: "deindent".to_string(),
            callback: |ui, _, _| {
                ui.ctx()
                    .push_markdown_event(Event::Indent { deindent: true })
            },
        }),
        Component::Button(ToolbarButton {
            icon: Icon::UNDO,
            id: "undo".to_string(),
            callback: |ui, _, _| ui.ctx().push_markdown_event(Event::Undo),
        }),
        Component::Button(ToolbarButton {
            icon: Icon::REDO,
            id: "redo".to_string(),
            callback: |ui, _, _| ui.ctx().push_markdown_event(Event::Undo),
        }),
    ]
}

/// increment from h1 -> h6 and then wraps back to h1
fn toggle_heading_style(
    ui: &mut egui::Ui, t: &mut ToolBar, _response: &mut markdown_editor::Response,
) {
    if t.header_click_count > 5 {
        t.header_click_count = 1;
    } else {
        t.header_click_count += 1;
    }
    ui.ctx()
        .push_markdown_event(Event::toggle_heading_style(t.header_click_count));
}
