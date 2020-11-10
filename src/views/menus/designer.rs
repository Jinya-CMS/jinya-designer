use jinya_ui::layout::page::Page;
use jinya_ui::widgets::dialog::confirmation::{ConfirmationDialog, DialogType};
use jinya_ui::widgets::toast::Toast;
use yew::agent::Dispatcher;
use yew::prelude::*;
use yew::services::fetch::*;

use crate::agents::menu_agent::{MenuAgent, MenuAgentRequest};
use crate::ajax::AjaxError;
use crate::ajax::menu_item_service::MenuItemService;
use crate::i18n::Translator;
use crate::models::menu_item::{MenuItem, SaveMenuItem};
use crate::views::menus::settings_dialog::{SettingsDialog, SettingsDialogType};

#[derive(PartialEq, Clone, Properties)]
pub struct MenuDesignerPageProps {
    pub id: usize,
}

pub enum Msg {
    OnMenuItemsLoaded(Result<Vec<MenuItem>, AjaxError>),
    OnIncreaseNesting(MouseEvent, Option<MenuItem>, MenuItem),
    OnDecreaseNesting(MouseEvent, MenuItem),
    OnRequestComplete,
    OnMenuItemDeleteClicked(MouseEvent, MenuItem),
    OnMenuItemEditClicked(MouseEvent, MenuItem),
    OnDeleteApprove,
    OnDeleteDecline,
    OnDeleteRequestComplete(Result<bool, AjaxError>),
    OnSaveMenuItemEdit(SaveMenuItem),
    OnDiscardMenuItemEdit,
    OnMenuItemSaved(Result<bool, AjaxError>),
    OnSaveMenuItemAdd(SaveMenuItem),
    OnDiscardMenuItemAdd,
    OnNewItemDragStart(DragEvent, SettingsDialogType),
    OnItemsDragEnter(DragEvent),
    OnItemsDragOver(DragEvent),
    OnDragOverItem(MenuItem, Option<usize>),
    OnDragOverLastItem(Option<usize>),
    OnItemDrop(DragEvent),
    OnMenuItemDragStart(DragEvent, MenuItem),
}

pub struct MenuDesignerPage {
    link: ComponentLink<Self>,
    id: usize,
    menu_items: Vec<MenuItem>,
    menu_item_service: MenuItemService,
    load_menu_items_task: Option<FetchTask>,
    translator: Translator,
    menu_dispatcher: Dispatcher<MenuAgent>,
    change_nesting_task: Option<FetchTask>,
    menu_item_to_delete: Option<MenuItem>,
    menu_item_delete_task: Option<FetchTask>,
    menu_item_update_task: Option<FetchTask>,
    menu_item_add_task: Option<FetchTask>,
    edit_menu_item_settings_type: Option<SettingsDialogType>,
    menu_item_to_edit: Option<MenuItem>,
    new_item_settings_type: Option<SettingsDialogType>,
    new_menu_item: Option<MenuItem>,
    selected_parent_item: Option<usize>,
    drag_over_item: Option<MenuItem>,
    first_item: bool,
    selected_menu_item: Option<MenuItem>,
}

impl Component for MenuDesignerPage {
    type Message = Msg;
    type Properties = MenuDesignerPageProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let menu_dispatcher = MenuAgent::dispatcher();
        let translator = Translator::new();

        MenuDesignerPage {
            link,
            id: props.id,
            menu_items: vec![],
            menu_item_service: MenuItemService::new(),
            load_menu_items_task: None,
            translator,
            menu_dispatcher,
            change_nesting_task: None,
            menu_item_to_delete: None,
            menu_item_delete_task: None,
            menu_item_update_task: None,
            menu_item_add_task: None,
            edit_menu_item_settings_type: None,
            menu_item_to_edit: None,
            new_item_settings_type: None,
            new_menu_item: None,
            selected_parent_item: None,
            drag_over_item: None,
            first_item: false,
            selected_menu_item: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::OnMenuItemsLoaded(result) => {
                if let Ok(data) = result {
                    self.menu_items = data;
                } else {
                    Toast::negative_toast(self.translator.translate("menus.designer.error_load_menu_items"))
                }
            }
            Msg::OnIncreaseNesting(event, previous, item) => {
                event.prevent_default();
                self.change_parent(previous, item);
            }
            Msg::OnDecreaseNesting(event, item) => {
                event.prevent_default();
                self.move_one_level_up(item);
            }
            Msg::OnRequestComplete => self.load_menu_items_task = Some(self.menu_item_service.get_by_menu(self.id, self.link.callback(Msg::OnMenuItemsLoaded))),
            Msg::OnMenuItemDeleteClicked(event, item) => {
                event.prevent_default();
                self.menu_item_to_delete = Some(item)
            }
            Msg::OnDeleteApprove => self.menu_item_delete_task = Some(self.menu_item_service.delete_menu_item(self.menu_item_to_delete.as_ref().unwrap().id, self.link.callback(Msg::OnDeleteRequestComplete))),
            Msg::OnDeleteDecline => self.menu_item_to_delete = None,
            Msg::OnDeleteRequestComplete(result) => {
                if result.is_ok() {
                    self.menu_item_to_delete = None;
                    self.load_menu_items_task = Some(self.menu_item_service.get_by_menu(self.id, self.link.callback(Msg::OnMenuItemsLoaded)));
                } else {
                    self.menu_item_to_delete = None;
                    Toast::negative_toast(self.translator.translate("menus.designer.item.delete.failed"));
                }
            }
            Msg::OnMenuItemEditClicked(event, item) => {
                event.prevent_default();
                self.menu_item_to_edit = Some(item.clone());
                self.edit_menu_item_settings_type = Some(if item.gallery.is_some() {
                    SettingsDialogType::Gallery
                } else if item.page.is_some() {
                    SettingsDialogType::Page
                } else if item.segment_page.is_some() {
                    SettingsDialogType::SegmentPage
                } else if item.artist.is_some() {
                    SettingsDialogType::ArtistProfile
                } else if item.route.is_some() {
                    SettingsDialogType::Link
                } else {
                    SettingsDialogType::Group
                });
            }
            Msg::OnSaveMenuItemEdit(result) => self.menu_item_update_task = Some(self.menu_item_service.update_menu_item(self.menu_item_to_edit.clone().unwrap().id, result, self.link.callback(Msg::OnMenuItemSaved))),
            Msg::OnDiscardMenuItemEdit => self.menu_item_to_edit = None,
            Msg::OnSaveMenuItemAdd(result) => {
                let mut item = result.clone();
                item.position = Some(if let Some(drag_over_item) = self.drag_over_item.as_ref() {
                    drag_over_item.position + 1
                } else {
                    0
                });
                if let Some(parent) = self.selected_parent_item.as_ref() {
                    self.menu_item_add_task = Some(self.menu_item_service.add_menu_item_by_parent(*parent, item, self.link.callback(Msg::OnMenuItemSaved)));
                } else {
                    self.menu_item_add_task = Some(self.menu_item_service.add_menu_item_by_menu(self.id, item, self.link.callback(Msg::OnMenuItemSaved)));
                }
            }
            Msg::OnDiscardMenuItemAdd => self.menu_item_to_edit = None,
            Msg::OnMenuItemSaved(result) => {
                self.menu_item_to_edit = None;
                self.new_menu_item = None;
                self.drag_over_item = None;
                self.first_item = false;
                self.selected_parent_item = None;
                if result.is_ok() {
                    self.load_menu_items_task = Some(self.menu_item_service.get_by_menu(self.id, self.link.callback(Msg::OnMenuItemsLoaded)))
                } else {
                    Toast::negative_toast(self.translator.translate("menus.designer.settings.error_save_settings_failed"))
                }
            }
            Msg::OnNewItemDragStart(event, item_type) => {
                if let Some(data_transfer) = event.data_transfer() {
                    data_transfer.set_drop_effect("copy");
                    data_transfer.set_effect_allowed("copy");
                    self.new_item_settings_type = Some(item_type);
                }
            }
            Msg::OnDragOverItem(item, parent_id) => {
                self.drag_over_item = Some(item);
                self.first_item = false;
                self.selected_parent_item = parent_id;
            }
            Msg::OnDragOverLastItem(parent_id) => {
                self.drag_over_item = None;
                self.first_item = true;
                self.selected_parent_item = parent_id;
            }
            Msg::OnItemsDragEnter(event) => {
                event.prevent_default();
                event.stop_propagation();
            }
            Msg::OnItemsDragOver(event) => {
                event.prevent_default();
                event.stop_propagation();
            }
            Msg::OnItemDrop(event) => {
                event.prevent_default();
                event.stop_propagation();
                if let Some(item) = self.selected_menu_item.as_ref() {
                    if let Some(parent) = self.selected_parent_item {
                        self.menu_item_update_task = Some(self.menu_item_service.change_menu_item_parent(parent, item.clone(), self.link.callback(Msg::OnMenuItemSaved)));
                    }
                } else {
                    self.new_menu_item = Some(MenuItem::empty());
                }
            }
            Msg::OnMenuItemDragStart(event, item) => {
                if let Some(data_transfer) = event.data_transfer() {
                    data_transfer.set_drop_effect("copy");
                    data_transfer.set_effect_allowed("copy");
                }
                self.selected_menu_item = Some(item);
            }
        }

        true
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        self.id = props.id;

        true
    }

    fn view(&self) -> Html {
        let drop_target_class = if self.first_item && self.selected_parent_item.is_none() {
            "jinya-designer-menu-item__drop-target jinya-designer-menu-item__drop-target--drag-over"
        } else {
            "jinya-designer-menu-item__drop-target"
        };
        html! {
            <Page>
                <div class="jinya-designer-menu-designer__container">
                    <div class="jinya-designer-menu-designer__list jinya-designer-menu-designer__list--new-items">
                        <div draggable=true ondragstart=self.link.callback(move |event| Msg::OnNewItemDragStart(event, SettingsDialogType::Gallery)) class="jinya-designer-menu-item__list-item">
                            <span class="mdi mdi-drag-horizontal-variant mdi-24px"></span> {self.translator.translate("menus.designer.gallery")}
                        </div>
                        <div draggable=true ondragstart=self.link.callback(move |event| Msg::OnNewItemDragStart(event, SettingsDialogType::Page)) class="jinya-designer-menu-item__list-item">
                            <span class="mdi mdi-drag-horizontal-variant mdi-24px"></span> {self.translator.translate("menus.designer.page")}
                        </div>
                        <div draggable=true ondragstart=self.link.callback(move |event| Msg::OnNewItemDragStart(event, SettingsDialogType::SegmentPage)) class="jinya-designer-menu-item__list-item">
                            <span class="mdi mdi-drag-horizontal-variant mdi-24px"></span> {self.translator.translate("menus.designer.segment_page")}
                        </div>
                        <div draggable=true ondragstart=self.link.callback(move |event| Msg::OnNewItemDragStart(event, SettingsDialogType::ArtistProfile)) class="jinya-designer-menu-item__list-item">
                            <span class="mdi mdi-drag-horizontal-variant mdi-24px"></span> {self.translator.translate("menus.designer.profile")}
                        </div>
                        <div draggable=true ondragstart=self.link.callback(move |event| Msg::OnNewItemDragStart(event, SettingsDialogType::Link)) class="jinya-designer-menu-item__list-item">
                            <span class="mdi mdi-drag-horizontal-variant mdi-24px"></span> {self.translator.translate("menus.designer.link")}
                        </div>
                        <div draggable=true ondragstart=self.link.callback(move |event| Msg::OnNewItemDragStart(event, SettingsDialogType::Group)) class="jinya-designer-menu-item__list-item">
                            <span class="mdi mdi-drag-horizontal-variant mdi-24px"></span> {self.translator.translate("menus.designer.group")}
                        </div>
                    </div>
                    <div ondragover=self.link.callback(|event| Msg::OnItemsDragOver(event)) ondragenter=self.link.callback(|event| Msg::OnItemsDragEnter(event)) class="jinya-designer-menu-designer__list jinya-designer-menu-designer__list--menu-items">
                        <div ondrop=self.link.callback(Msg::OnItemDrop) ondragover=self.link.callback(move |event| Msg::OnDragOverLastItem(None)) class=drop_target_class>
                            <span class="mdi mdi-plus jinya-designer-menu-item__drop-target-icon"></span>
                        </div>
                        <ul class="jinya-designer-menu-designer__items jinya-designer-menu-designer__items--top">
                            {for self.menu_items.iter().enumerate().map(|(idx, item)| {
                                let previous = if idx > 0 {
                                    Some(self.menu_items[idx - 1].clone())
                                } else {
                                    None
                                };
                                html! {
                                    {self.get_item_view(item, None, idx == 0, previous, idx + 1 == item.items.len())}
                                }
                            })}
                        </ul>
                    </div>
                </div>
                {if self.menu_item_to_delete.is_some() {
                    let item = self.menu_item_to_delete.as_ref().unwrap();
                    html! {
                        <ConfirmationDialog
                            title=self.translator.translate("menus.designer.item.delete.title")
                            dialog_type=DialogType::Negative
                            message=self.translator.translate_with_args("menus.designer.item.delete.content", map! { "title" => item.title.as_str() })
                            decline_label=self.translator.translate("menus.designer.item.delete.decline")
                            approve_label=self.translator.translate("menus.designer.item.delete.approve")
                            on_approve=self.link.callback(|_| Msg::OnDeleteApprove)
                            on_decline=self.link.callback(|_| Msg::OnDeleteDecline)
                            is_open=self.menu_item_to_delete.is_some()
                        />
                    }
                } else {
                    html! {}
                }}
                {if self.new_menu_item.is_some() {
                    let item = self.new_menu_item.as_ref().unwrap();
                    html! {
                        <SettingsDialog
                            is_open=true
                            dialog_type=self.new_item_settings_type.as_ref().unwrap()
                            on_save_changes=self.link.callback(Msg::OnSaveMenuItemAdd)
                            on_discard_changes=self.link.callback(|_| Msg::OnDiscardMenuItemAdd)
                            menu_item=item
                        />
                    }
                } else {
                    html! {}
                }}
                {if self.menu_item_to_edit.is_some() {
                    let item = self.menu_item_to_edit.as_ref().unwrap();
                    html! {
                        <SettingsDialog
                            is_open=self.menu_item_to_edit.is_some()
                            dialog_type=self.edit_menu_item_settings_type.as_ref().unwrap()
                            on_save_changes=self.link.callback(Msg::OnSaveMenuItemEdit)
                            on_discard_changes=self.link.callback(|_| Msg::OnDiscardMenuItemEdit)
                            menu_item=item
                        />
                    }
                } else {
                    html! {}
                }}
            </Page>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.menu_dispatcher.send(MenuAgentRequest::ChangeTitle(self.translator.translate("app.menu.content.pages.segment_pages")));
            self.menu_dispatcher.send(MenuAgentRequest::HideSearch);
            self.load_menu_items_task = Some(self.menu_item_service.get_by_menu(self.id, self.link.callback(Msg::OnMenuItemsLoaded)));
        }
    }
}

impl MenuDesignerPage {
    fn get_item_view(&self, item: &MenuItem, parent: Option<MenuItem>, first: bool, previous: Option<MenuItem>, last: bool) -> Html {
        let delete_item = item.clone();
        let edit_item = item.clone();
        let drop_item = item.clone();
        let drag_start_item = item.clone();
        let drop_target_class = if let Some(drag_over_item) = self.drag_over_item.as_ref() {
            if drag_over_item.id == item.id {
                "jinya-designer-menu-item__drop-target jinya-designer-menu-item__drop-target--drag-over"
            } else {
                "jinya-designer-menu-item__drop-target"
            }
        } else {
            "jinya-designer-menu-item__drop-target"
        };
        let parent_id = if let Some(parent_item) = parent.clone() {
            Some(parent_item.id)
        } else {
            None
        };
        html! {
            <>
                <li>
                    <div ondragstart=self.link.callback(move |event| Msg::OnMenuItemDragStart(event, drag_start_item.clone())) class="jinya-designer-menu-item__list-item" draggable=true>
                        <div style="display: flex">
                            <span class="mdi mdi-drag-horizontal-variant mdi-24px"></span>
                            <span>
                                {&item.title} {if item.highlighted {
                                    " (hightlighted) "
                                } else {
                                    " "
                                }}
                            </span>
                            {if item.route.is_some() {
                                html! {
                                    <span class="jinya-designer-menu-item__link">{"/"}{&item.route.as_ref().unwrap()}</span>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                        <div class="jinya-designer-menu-item__button-row">
                            {if first {
                                html! {}
                            } else if parent.is_none() {
                                let cloned_item = item.clone();
                                html! {
                                    <a onclick=self.link.callback(move |event| Msg::OnIncreaseNesting(event, previous.clone(), cloned_item.clone())) class="jinya-designer-menu-item__button jinya-designer-menu-item__button--primary mdi mdi-chevron-right"></a>
                                }
                            } else if item.items.is_empty() {
                                html! {
                                    <>
                                        {if parent.is_some() && last {
                                            let cloned_item = item.clone();
                                            html! {
                                                <a onclick=self.link.callback(move |event| Msg::OnDecreaseNesting(event, cloned_item.clone())) class="jinya-designer-menu-item__button jinya-designer-menu-item__button--primary mdi mdi-chevron-left"></a>
                                            }
                                        } else {
                                            html! {}
                                        }}
                                        {if previous.is_some() {
                                            let cloned_item = item.clone();
                                            html! {
                                                <a onclick=self.link.callback(move |event| Msg::OnIncreaseNesting(event, previous.clone(), cloned_item.clone())) class="jinya-designer-menu-item__button jinya-designer-menu-item__button--primary mdi mdi-chevron-right"></a>
                                            }
                                        } else {
                                            html! {}
                                        }}
                                    </>
                                }
                            } else {
                                html! {}
                            }}
                            <a onclick=self.link.callback(move |event| Msg::OnMenuItemEditClicked(event, edit_item.clone())) class="jinya-designer-menu-item__button jinya-designer-menu-item__button--primary mdi mdi-pencil"></a>
                            <a onclick=self.link.callback(move |event| Msg::OnMenuItemDeleteClicked(event, delete_item.clone())) class="jinya-designer-menu-item__button jinya-designer-menu-item__button--negative mdi mdi-delete"></a>
                        </div>
                    </div>
                    <div ondrop=self.link.callback(Msg::OnItemDrop) ondragover=self.link.callback(move |event| Msg::OnDragOverItem(drop_item.clone(), parent_id.clone())) class=drop_target_class>
                        <span class="mdi mdi-plus jinya-designer-menu-item__drop-target-icon"></span>
                    </div>
                    {if !item.items.is_empty() {
                        html! {
                            <ul class="jinya-designer-menu-designer__items">
                                {for item.items.iter().enumerate().map(|(idx, subitem)| {
                                    let previous = if idx > 0 {
                                        Some(item.items[idx - 1].clone())
                                    } else {
                                        None
                                    };
                                    html! {
                                        {self.get_item_view(subitem, Some(item.clone()), false, previous, idx + 1 == item.items.len())}
                                    }
                                })}
                            </ul>
                        }
                    } else {
                        html! {}
                    }}
                </li>
            </>
        }
    }

    fn change_parent(&mut self, new_parent: Option<MenuItem>, item: MenuItem) {
        if let Some(new_parent_item) = new_parent {
            self.change_nesting_task = Some(self.menu_item_service.change_menu_item_parent(new_parent_item.id, item, self.link.callback(|_| Msg::OnRequestComplete)));
        }
    }

    fn move_one_level_up(&mut self, item: MenuItem) {
        self.change_nesting_task = Some(self.menu_item_service.move_item_one_level_up(self.id, item, self.link.callback(|_| Msg::OnRequestComplete)));
    }
}