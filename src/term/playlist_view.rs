use crossterm::event::{KeyCode, KeyEvent};
use flume::Sender;
use ratatui::{layout::Rect, style::Style, Frame};
use ytpapi2::YoutubeMusicVideoRef;

use crate::{consts::CONFIG, structures::sound_action::SoundAction, utils::invert, DATABASE};

use super::{
    item_list::{ListItem, ListItemAction},
    styles::{
        STYLE_PLAYLIST_ITEM_CURRENT_DOWNLOADING, STYLE_PLAYLIST_ITEM_DOWNLOADING,
        STYLE_PLAYLIST_ITEM_LOCAL, STYLE_PLAYLIST_ITEM_SELECTED_DOWNLOADING,
        STYLE_PLAYLIST_ITEM_SELECTED_LOCAL,
    },
    EventResponse, ManagerMessage, Screen, Screens,
};

#[derive(Clone)]
pub struct PlayListAction(usize, bool);

impl ListItemAction for PlayListAction {
    fn render_style(&self, _: &str, selected: bool) -> Style {
        if selected {
            if self.1 {
                *STYLE_PLAYLIST_ITEM_SELECTED_DOWNLOADING
            } else {
                *STYLE_PLAYLIST_ITEM_SELECTED_LOCAL
            }
        } else if self.1 {
            *STYLE_PLAYLIST_ITEM_DOWNLOADING
        } else {
            *STYLE_PLAYLIST_ITEM_LOCAL
        }
    }
}

// Audio device not connected!
pub struct PlaylistView {
    pub items: ListItem<PlayListAction>,
    pub videos: Vec<YoutubeMusicVideoRef>,
    pub goto: Screens,
    pub sender: Sender<SoundAction>,
}

impl Screen for PlaylistView {
    fn on_mouse_press(&mut self, e: crossterm::event::MouseEvent, r: &Rect) -> EventResponse {
        if let Some(PlayListAction(v, _)) = self.items.on_mouse_press(e, r) {
            self.sender
                .send(SoundAction::ReplaceQueue(
                    self.videos.iter().skip(v).cloned().collect(),
                ))
                .unwrap();
            EventResponse::Message(vec![ManagerMessage::PlayerFrom(Screens::Playlist)])
        } else {
            EventResponse::None
        }
    }

    fn on_key_press(&mut self, key: KeyEvent, _: &Rect) -> EventResponse {
        if let Some(PlayListAction(v, _)) = self.items.on_key_press(key) {
            self.sender
                .send(SoundAction::ReplaceQueue(
                    self.videos.iter().skip(*v).cloned().collect(),
                ))
                .unwrap();
            return EventResponse::Message(vec![ManagerMessage::PlayerFrom(Screens::Playlist)]);
        }
        match key.code {
            KeyCode::Esc => ManagerMessage::ChangeState(self.goto).event(),
            KeyCode::Char('f') => ManagerMessage::SearchFrom(Screens::PlaylistViewer).event(),
            _ => EventResponse::None,
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(&self.items, frame.size());
    }

    fn handle_global_message(&mut self, m: ManagerMessage) -> EventResponse {
        match m {
            ManagerMessage::Inspect(a, screen, m) => {
                self.items.set_title(format!(" Inspecting {a} "));
                self.goto = screen;
                let db = DATABASE.read().unwrap();
                self.items.update(
                    m.iter()
                        .enumerate()
                        .map(|(i, m)| {
                            (
                                format!("  {m}"),
                                PlayListAction(i, !db.iter().any(|x| x.video_id == m.video_id)),
                            )
                        })
                        .collect(),
                    0,
                );
                self.videos = m;

                EventResponse::Message(vec![ManagerMessage::ChangeState(Screens::PlaylistViewer)])
            }
            _ => EventResponse::None,
        }
    }

    fn close(&mut self, _: Screens) -> EventResponse {
        EventResponse::None
    }

    fn open(&mut self) -> EventResponse {
        EventResponse::None
    }
}
