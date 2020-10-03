use std::str::FromStr;

use jinya_ui::layout::button_row::ButtonRow;
use jinya_ui::layout::page::Page;
use jinya_ui::layout::row::Row;
use jinya_ui::widgets::button::{Button, ButtonType};
use jinya_ui::widgets::dialog::confirmation::{ConfirmationDialog, DialogType};
use jinya_ui::widgets::form::dropdown::{Dropdown, DropdownItem};
use jinya_ui::widgets::toast::Toast;
use web_sys::Node;
use yew::prelude::*;
use yew::services::fetch::*;
use yew::virtual_dom::VNode;

use crate::ajax::{AjaxError, get_host};
use crate::ajax::file_service::FileService;
use crate::ajax::gallery_service::GalleryService;
use crate::ajax::segment_service::SegmentService;
use crate::i18n::Translator;
use crate::models::file::File;
use crate::models::gallery::Gallery;
use crate::models::list_model::ListModel;
use crate::models::segment::Segment;

#[derive(PartialEq, Clone, Properties)]
pub struct SegmentPageDesignerPageProps {
    pub id: usize,
}

pub enum Msg {
    OnSegmentsLoaded(Result<Vec<Segment>, AjaxError>),
    OnFilesLoaded(Result<ListModel<File>, AjaxError>),
    OnGalleriesLoaded(Result<ListModel<Gallery>, AjaxError>),
    OnEditSegment(MouseEvent, usize),
    OnDeleteSegment(MouseEvent, usize),
    OnDeleteApprove,
    OnDeleteDecline,
    OnDeleteRequestCompleted(Result<bool, AjaxError>),
    OnUpdateGallerySegmentRequestCompleted(Result<bool, AjaxError>),
    OnGallerySelect(String),
    OnUpdateGallerySegment,
    OnDiscardUpdateGallerySegment,
}

pub struct SegmentPageDesignerPage {
    link: ComponentLink<Self>,
    id: usize,
    segments: Vec<Segment>,
    delete_segment_task: Option<FetchTask>,
    load_segments_task: Option<FetchTask>,
    load_files_task: Option<FetchTask>,
    load_galleries_task: Option<FetchTask>,
    update_gallery_segment_task: Option<FetchTask>,
    segment_service: SegmentService,
    file_service: FileService,
    gallery_service: GalleryService,
    translator: Translator,
    files: Vec<File>,
    galleries: Vec<DropdownItem>,
    segment_to_delete: Option<Segment>,
    segment_to_edit: Option<Segment>,
    edit_segment_gallery_name: Option<String>,
    fetched_galleries: Vec<Gallery>,
}

impl Component for SegmentPageDesignerPage {
    type Message = Msg;
    type Properties = SegmentPageDesignerPageProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        SegmentPageDesignerPage {
            link,
            id: props.id,
            segments: vec![],
            delete_segment_task: None,
            load_segments_task: None,
            load_files_task: None,
            load_galleries_task: None,
            update_gallery_segment_task: None,
            segment_service: SegmentService::new(),
            file_service: FileService::new(),
            gallery_service: GalleryService::new(),
            translator: Translator::new(),
            files: vec![],
            galleries: vec![],
            segment_to_delete: None,
            segment_to_edit: None,
            edit_segment_gallery_name: None,
            fetched_galleries: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::OnSegmentsLoaded(data) => {
                if data.is_ok() {
                    self.segments = data.unwrap()
                } else {
                    Toast::negative_toast(self.translator.translate("segment_pages.designer.error_load_segments"))
                }
            }
            Msg::OnFilesLoaded(data) => {
                if data.is_ok() {
                    self.files = data.unwrap().items
                } else {
                    Toast::negative_toast(self.translator.translate("segment_pages.designer.error_load_files"))
                }
            }
            Msg::OnGalleriesLoaded(data) => {
                if data.is_ok() {
                    let items = data.unwrap().items;
                    self.galleries = items.iter().map(|item| {
                        DropdownItem {
                            value: item.name.to_string(),
                            text: item.name.to_string(),
                        }
                    }).collect();
                    self.fetched_galleries = items;
                } else {
                    Toast::negative_toast(self.translator.translate("segment_pages.designer.error_load_galleries"))
                }
            }
            Msg::OnDeleteSegment(event, idx) => {
                event.prevent_default();
                self.segment_to_delete = Some(self.segments[idx].clone());
            }
            Msg::OnDeleteApprove => self.delete_segment_task = Some(self.segment_service.delete_segment(self.id, self.segment_to_delete.as_ref().unwrap().position, self.link.callback(|result| Msg::OnDeleteRequestCompleted(result)))),
            Msg::OnDeleteDecline => self.segment_to_delete = None,
            Msg::OnDeleteRequestCompleted(result) => {
                self.segment_to_delete = None;
                if result.is_err() {
                    Toast::negative_toast(self.translator.translate("segment_pages.designer.delete.failed"))
                } else {
                    self.load_segments_task = Some(self.segment_service.get_segments(self.id, self.link.callback(|data| Msg::OnSegmentsLoaded(data))));
                }
            }
            Msg::OnEditSegment(event, idx) => {
                event.prevent_default();
                self.segment_to_edit = Some(self.segments[idx].clone());
                if self.segment_to_edit.as_ref().unwrap().gallery.is_some() {
                    let segment = self.segment_to_edit.as_ref().unwrap();
                    let gallery = segment.gallery.as_ref().unwrap();
                    self.edit_segment_gallery_name = Some(gallery.name.to_string());
                    self.load_galleries_task = Some(self.gallery_service.get_list("".to_string(), self.link.callback(|result| Msg::OnGalleriesLoaded(result))));
                }
            }
            Msg::OnGallerySelect(item) => self.edit_segment_gallery_name = Some(item),
            Msg::OnUpdateGallerySegment => {
                let segment_to_edit = self.segment_to_edit.as_ref().unwrap();
                let name = self.edit_segment_gallery_name.as_ref().unwrap();
                let gallery = self.fetched_galleries.iter().find(move |item| item.name.eq(name));
                if gallery.is_some() {
                    self.update_gallery_segment_task = Some(self.segment_service.update_gallery_segment(self.id, segment_to_edit.position, gallery.unwrap().id, self.link.callback(|result| Msg::OnUpdateGallerySegmentRequestCompleted(result))));
                }
            }
            Msg::OnDiscardUpdateGallerySegment => self.segment_to_edit = None,
            Msg::OnUpdateGallerySegmentRequestCompleted(result) => {
                if result.is_ok() {
                    self.segment_to_edit = None;
                    self.edit_segment_gallery_name = None;
                    self.load_segments_task = Some(self.segment_service.get_segments(self.id, self.link.callback(|data| Msg::OnSegmentsLoaded(data))));
                } else {
                    Toast::negative_toast(self.translator.translate("segment_pages.designer.error_change_gallery_failed"))
                }
            }
        }

        true
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        self.id = props.id;

        true
    }

    fn view(&self) -> Html {
        html! {
            <Page>
                <div class="jinya-designer-segment-page-designer__container">
                    <div class="jinya-designer-segment-page-designer__list jinya-designer-segment-page-designer__list--new-items">
                        <div class="jinya-designer-segment__list-item jinya-designer-segment__list-item--gallery">
                            <span class="mdi mdi-menu mdi-24px"></span>  {self.translator.translate("segment_pages.designer.gallery")}
                        </div>
                        <div class="jinya-designer-segment__list-item jinya-designer-segment__list-item--file">
                            <span class="mdi mdi-menu mdi-24px"></span> {self.translator.translate("segment_pages.designer.file")}
                        </div>
                        <div class="jinya-designer-segment__list-item jinya-designer-segment__list-item--html">
                            <span class="mdi mdi-menu mdi-24px"></span> {self.translator.translate("segment_pages.designer.html")}
                        </div>
                    </div>
                    <div class="jinya-designer-segment-page-designer__list jinya-designer-segment-page-designer__list--segments">
                        {for self.segments.iter().enumerate().map(|(idx, item)| {
                            if item.gallery.is_some() {
                                let gallery = item.gallery.as_ref().unwrap();
                                html! {
                                    <div class="jinya-designer-segment jinya-designer-segment--gallery">
                                        <a onclick=self.link.callback(move |event| Msg::OnDeleteSegment(event, idx)) class="mdi mdi-delete jinya-designer-segment__button jinya-designer-segment__button--negative"></a>
                                        <a onclick=self.link.callback(move |event| Msg::OnEditSegment(event, idx)) class="mdi mdi-pencil jinya-designer-segment__button jinya-designer-segment__button--primary"></a>
                                        <span class="jinya-designer-segment__title jinya-designer-segment__title--gallery">{self.translator.translate("segment_pages.designer.gallery")}</span>
                                        {if self.segment_to_edit.is_some() && self.segment_to_edit.as_ref().unwrap().id == item.id {
                                            html! {
                                                <>
                                                    <Row>
                                                        <Dropdown
                                                            label=self.translator.translate("segment_pages.designer.gallery")
                                                            on_select=self.link.callback(|value| Msg::OnGallerySelect(value))
                                                            items=&self.galleries
                                                            value=self.edit_segment_gallery_name.as_ref().unwrap()
                                                        />
                                                    </Row>
                                                    <ButtonRow>
                                                        <Button label=self.translator.translate("segment_pages.designer.action_save_gallery") button_type=ButtonType::Primary on_click=self.link.callback(|_| Msg::OnUpdateGallerySegment)></Button>
                                                        <Button label=self.translator.translate("segment_pages.designer.action_discard_gallery") button_type=ButtonType::Secondary on_click=self.link.callback(|_| Msg::OnDiscardUpdateGallerySegment)></Button>
                                                    </ButtonRow>
                                                </>
                                            }
                                        } else {
                                            html! {
                                                <span>{&gallery.name}</span>
                                            }
                                        }}
                                    </div>
                                }
                            } else if item.file.is_some() {
                                let file = item.file.as_ref().unwrap();
                                let action = if item.action.is_some() {
                                    format!("segment_pages.designer.action.{}", item.action.as_ref().unwrap().to_string())
                                } else {
                                    "–".to_string()
                                };
                                let target = if item.target.is_some() {
                                    item.target.as_ref().unwrap().to_string()
                                } else {
                                    "–".to_string()
                                };
                                let script = if item.script.is_some() {
                                    item.script.as_ref().unwrap().to_string()
                                } else {
                                    "–".to_string()
                                };
                                html! {
                                    <div class="jinya-designer-segment jinya-designer-segment--file">
                                        <img class="jinya-designer-segment__image" src={format!("{}{}", get_host(), &file.path)} />
                                        <div class="jinya-designer-segment__modifiers">
                                            <a onclick=self.link.callback(move |event| Msg::OnDeleteSegment(event, idx)) class="mdi mdi-delete jinya-designer-segment__button jinya-designer-segment__button--negative"></a>
                                            <a class="mdi mdi-pencil jinya-designer-segment__button jinya-designer-segment__button--primary"></a>
                                            <dl>
                                                <dt class="jinya-designer-segment__action-header">{self.translator.translate("segment_pages.designer.action")}</dt>
                                                <dd class="jinya-designer-segment__action-value">{self.translator.translate(action.as_str())}</dd>
                                                <dt class="jinya-designer-segment__target-header">{self.translator.translate("segment_pages.designer.target")}</dt>
                                                <dd class="jinya-designer-segment__target-value">{&target}</dd>
                                                <dt class="jinya-designer-segment__script-header">{self.translator.translate("segment_pages.designer.script")}</dt>
                                                <dd class="jinya-designer-segment__script-value"><pre>{&script}</pre></dd>
                                            </dl>
                                        </div>
                                    </div>
                                }
                            } else if item.html.is_some() {
                                let html = item.html.as_ref().unwrap();
                                let content = {
                                    let div = web_sys::window()
                                        .unwrap()
                                        .document()
                                        .unwrap()
                                        .create_element("div")
                                        .unwrap();
                                    div.set_inner_html(html);
                                    div
                                };
                                let node = Node::from(content);
                                let vnode = VNode::VRef(node);
                                html! {
                                    <div class="jinya-designer-segment jinya-designer-segment--html">
                                        <a onclick=self.link.callback(move |event| Msg::OnDeleteSegment(event, idx)) class="mdi mdi-delete jinya-designer-segment__button jinya-designer-segment__button--negative"></a>
                                        <a class="mdi mdi-pencil jinya-designer-segment__button jinya-designer-segment__button--primary"></a>
                                        <span class="jinya-designer-segment__title jinya-designer-segment__title--gallery">{self.translator.translate("segment_pages.designer.html")}</span>
                                        {vnode}
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        })}
                    </div>
                </div>
                {if self.segment_to_delete.is_some() {
                    html! {
                        <ConfirmationDialog
                            title=self.translator.translate("segment_pages.designer.delete.title")
                            dialog_type=DialogType::Negative
                            message=self.translator.translate("segment_pages.designer.delete.content")
                            decline_label=self.translator.translate("segment_pages.designer.delete.decline")
                            approve_label=self.translator.translate("segment_pages.designer.delete.approve")
                            on_approve=self.link.callback(|_| Msg::OnDeleteApprove)
                            on_decline=self.link.callback(|_| Msg::OnDeleteDecline)
                            is_open=self.segment_to_delete.is_some()
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
            self.load_segments_task = Some(self.segment_service.get_segments(self.id, self.link.callback(|data| Msg::OnSegmentsLoaded(data))));
        }
    }
}