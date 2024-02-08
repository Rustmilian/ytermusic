use cssparser::get_style;
use once_cell::sync::Lazy;
use ratatui::style::Style;

pub static STYLE_PLAYLIST_ITEM_SELECTED_DOWNLOADING: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["selected", "downloading"])
});
pub static STYLE_PLAYLIST_ITEM_SELECTED_LOCAL: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["selected", "local"])
});
pub static STYLE_PLAYLIST_ITEM_SELECTED_WAITING: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["selected", "waiting"])
});
pub static STYLE_PLAYLIST_ITEM_SELECTED_ERROR: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["selected", "error"])
});
pub static STYLE_PLAYLIST_ITEM_DOWNLOADING: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["downloading"])
});
pub static STYLE_PLAYLIST_ITEM_LOCAL: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["local"])
});
pub static STYLE_PLAYLIST_ITEM_WAITING: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["waiting"])
});
pub static STYLE_PLAYLIST_ITEM_ERROR: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["error"])
});

pub static STYLE_PLAYLIST_ITEM_CURRENT_PLAYING: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["playing","current"])
});
pub static STYLE_PLAYLIST_ITEM_CURRENT_PAUSED: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["paused","current"])
});
pub static STYLE_PLAYLIST_ITEM_CURRENT_DOWNLOADING: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["downloading","current"])
});
pub static STYLE_PLAYLIST_ITEM_CURRENT_ERROR: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-item", &["error","current"])
});

pub static STYLE_PLAYLIST_LIST_ITEM: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-list", &["item"])
});
pub static STYLE_PLAYLIST_LIST_ITEM_SELECTED: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-list", &["item", "selected"])
});
pub static STYLE_PLAYLIST_LIST_ITEM_LOCAL_SELECTED: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-list", &["item", "local", "selected"])
});
pub static STYLE_PLAYLIST_LIST_ITEM_LOCAL: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-list", &["item", "local"])
});
pub static STYLE_PLAYLIST_LIST_ITEM_LAST: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-list", &["item", "last"])
});
pub static STYLE_PLAYLIST_LIST_ITEM_LAST_SELECTED: Lazy<Style> = Lazy::new(|| {
    get_style("playlist-list", &["item", "last", "selected"])
});