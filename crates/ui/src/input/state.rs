//! Input state management for text editing.

use std::ops::Range;
use std::time::{Duration, Instant};

use gpui::{
    point, px, App, AppContext, Bounds, ClipboardItem, Context, Entity, EventEmitter, FocusHandle,
    Focusable, Pixels, Point, SharedString, Subscription, TextRun, TextStyle, UTF16Selection,
    Window, WrappedLine,
};

use super::bidi::{detect_base_direction, TextDirection};
use super::bindings::{
    Backspace, Copy, Cut, Delete, DeleteToBeginningOfLine, DeleteToEndOfLine, DeleteWordLeft,
    DeleteWordRight, Down, End, Enter, Home, Left, MoveToBeginning, MoveToEnd, Paste, Redo, Right,
    SelectAll, SelectDown, SelectLeft, SelectRight, SelectToBeginning, SelectToEnd, SelectUp,
    SelectWordLeft, SelectWordRight, Tab, Undo, Up, WordLeft, WordRight,
};
use super::blink::CursorBlink;
use super::handler::EntityInputHandler;
use unicode_segmentation::UnicodeSegmentation;

const DEFAULT_GROUP_INTERVAL: Duration = Duration::from_millis(300);
const DEFAULT_BLINK_INTERVAL: Duration = Duration::from_millis(500);
const MAX_HISTORY_LEN: usize = 1000;

/// Events emitted by InputState when significant changes occur.
#[derive(Clone, Debug)]
pub enum InputStateEvent {
    Focus,
    Blur,
    TextChanged,
    Undo,
    Redo,
}

impl EventEmitter<InputStateEvent> for InputState {}

#[derive(Clone, Debug)]
struct HistoryEntry {
    range: Range<usize>,
    old_text: String,
    new_text_len: usize,
    selected_range: Range<usize>,
    selection_reversed: bool,
    timestamp: Instant,
}

impl HistoryEntry {
    fn apply_undo(&self, content: &mut String) -> HistoryEntry {
        let undo_start = self.range.start;
        let undo_end = (self.range.start + self.new_text_len).min(content.len());
        let removed_text = content[undo_start..undo_end].to_string();
        content.replace_range(undo_start..undo_end, &self.old_text);

        HistoryEntry {
            range: undo_start..undo_start + self.old_text.len(),
            old_text: removed_text,
            new_text_len: self.old_text.len(),
            selected_range: self.selected_range.clone(),
            selection_reversed: self.selection_reversed,
            timestamp: self.timestamp,
        }
    }

    fn apply_redo(&self, content: &mut String) -> HistoryEntry {
        self.apply_undo(content)
    }
}

/// Input state for text editing.
pub struct InputState {
    focus_handle: FocusHandle,
    content: String,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    pub(crate) line_height: Pixels,
    pub(crate) line_layouts: Vec<InputLineLayout>,
    pub(crate) wrap_width: Option<Pixels>,
    pub(crate) text_style: Option<TextStyle>,
    pub(crate) needs_layout: bool,
    is_selecting: bool,
    last_click_position: Option<Point<Pixels>>,
    click_count: usize,
    pub(crate) scroll_offset: Pixels,
    pub(crate) available_height: Pixels,
    pub(crate) available_width: Pixels,
    multiline: bool,
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
    group_interval: Duration,
    cursor_blink: Option<Entity<CursorBlink>>,
    _subscriptions: Vec<Subscription>,
    was_focused: bool,
    cached_utf16_len: Option<usize>,
}

/// Layout information for a single logical line of text.
#[derive(Clone, Debug)]
pub struct InputLineLayout {
    pub text_range: Range<usize>,
    pub wrapped_line: Option<WrappedLine>,
    pub y_offset: Pixels,
    pub visual_line_count: usize,
    pub direction: TextDirection,
}

impl InputState {
    /// Creates a new multiline InputState.
    pub fn new_multiline(cx: &mut Context<Self>) -> Self {
        Self::new(cx).multiline(true)
    }

    /// Creates a new singleline InputState.
    pub fn new_singleline(cx: &mut Context<Self>) -> Self {
        Self::new(cx).multiline(false)
    }

    /// Creates a new InputState.
    pub fn new(cx: &mut Context<Self>) -> Self {
        let cursor_blink = cx.new(|cx| CursorBlink::new(DEFAULT_BLINK_INTERVAL, cx));
        let blink_subscription = cx.observe(&cursor_blink, |_, _, cx| cx.notify());

        Self {
            focus_handle: cx.focus_handle(),
            content: String::new(),
            placeholder: SharedString::default(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            line_height: px(0.),
            line_layouts: Vec::new(),
            wrap_width: None,
            text_style: None,
            needs_layout: true,
            is_selecting: false,
            last_click_position: None,
            click_count: 0,
            scroll_offset: px(0.),
            available_height: px(0.),
            available_width: px(0.),
            multiline: false,
            undo_stack: Vec::new(),
            cached_utf16_len: None,
            redo_stack: Vec::new(),
            group_interval: DEFAULT_GROUP_INTERVAL,
            cursor_blink: Some(cursor_blink),
            _subscriptions: vec![blink_subscription],
            was_focused: false,
        }
    }

    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    pub fn is_multiline(&self) -> bool {
        self.multiline
    }

    pub fn cursor_blink(mut self, enabled: bool) -> Self {
        if !enabled {
            self.cursor_blink = None;
        }
        self
    }

    pub fn cursor_visible(&mut self, is_focused: bool, cx: &mut Context<Self>) -> bool {
        if let Some(cursor_blink) = &self.cursor_blink {
            if is_focused && !self.was_focused {
                cursor_blink.update(cx, |cb, cx| cb.enable(cx));
                cx.emit(InputStateEvent::Focus);
            } else if !is_focused && self.was_focused {
                cursor_blink.update(cx, |cb, cx| cb.disable(cx));
                cx.emit(InputStateEvent::Blur);
            }
        }
        self.was_focused = is_focused;

        self.cursor_blink
            .as_ref()
            .map(|cb| cb.read(cx).visible())
            .unwrap_or(true)
    }

    fn pause_cursor_blink(&self, cx: &mut Context<Self>) {
        if let Some(cursor_blink) = &self.cursor_blink {
            cursor_blink.update(cx, |cb, cx| cb.pause_blinking(cx));
        }
    }

    pub(crate) fn set_text_style(&mut self, style: &TextStyle) {
        let changed = self
            .text_style
            .as_ref()
            .map_or(true, |current| current != style);

        if changed {
            self.text_style = Some(style.clone());
            self.needs_layout = true;
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content(&mut self, content: impl Into<String>, cx: &mut Context<Self>) {
        let content = content.into();
        self.content = if self.multiline {
            content
        } else {
            content.replace('\n', " ").replace('\r', "")
        };
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        self.needs_layout = true;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.cached_utf16_len = None;
        self.pause_cursor_blink(cx);
        cx.emit(InputStateEvent::TextChanged);
        cx.notify();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn push_undo_patch(&mut self, range: Range<usize>, new_text_len: usize) {
        if self.marked_range.is_some() {
            return;
        }

        let now = Instant::now();

        if let Some(last) = self.undo_stack.last() {
            if now.duration_since(last.timestamp) < self.group_interval {
                return;
            }
        }

        let old_text = self.content[range.clone()].to_string();

        self.undo_stack.push(HistoryEntry {
            range: range.start..range.start + new_text_len,
            old_text,
            new_text_len,
            selected_range: self.selected_range.clone(),
            selection_reversed: self.selection_reversed,
            timestamp: now,
        });

        if self.undo_stack.len() > MAX_HISTORY_LEN {
            self.undo_stack.remove(0);
        }

        self.redo_stack.clear();
    }

    pub(crate) fn undo(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(entry) = self.undo_stack.pop() {
            let selected_range = entry.selected_range.clone();
            let selection_reversed = entry.selection_reversed;

            let redo_entry = entry.apply_undo(&mut self.content);
            self.redo_stack.push(redo_entry);

            self.selected_range = selected_range;
            self.selection_reversed = selection_reversed;
            self.needs_layout = true;
            self.cached_utf16_len = None;
            self.scroll_to_cursor();
            cx.emit(InputStateEvent::Undo);
            cx.notify();
        }
    }

    pub(crate) fn redo(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(entry) = self.redo_stack.pop() {
            let undo_entry = entry.apply_redo(&mut self.content);

            let cursor_pos = undo_entry.range.start;
            self.selected_range = cursor_pos..cursor_pos;
            self.selection_reversed = false;

            self.undo_stack.push(undo_entry);
            self.needs_layout = true;
            self.cached_utf16_len = None;
            self.scroll_to_cursor();
            cx.emit(InputStateEvent::Redo);
            cx.notify();
        }
    }

    pub fn placeholder(&self) -> &SharedString {
        &self.placeholder
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.placeholder = placeholder.into();
        cx.notify();
    }

    pub fn selected_range(&self) -> &Range<usize> {
        &self.selected_range
    }

    pub fn selection_reversed(&self) -> bool {
        self.selection_reversed
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub fn marked_range(&self) -> Option<&Range<usize>> {
        self.marked_range.as_ref()
    }

    pub fn set_selected_range(&mut self, range: Range<usize>) {
        let range = range.start.min(self.content.len())..range.end.min(self.content.len());
        self.selected_range = range;
        self.selection_reversed = false;
    }

    pub fn selected_text_range_utf16(&self) -> Range<usize> {
        self.range_to_utf16(&self.selected_range)
    }

    pub fn insert_text(&mut self, text: &str, cx: &mut Context<Self>) {
        let range = self
            .marked_range
            .clone()
            .unwrap_or(self.selected_range.clone());
        let range = range.start.min(self.content.len())..range.end.min(self.content.len());

        let sanitized_text;
        let text_to_insert = if self.multiline {
            text
        } else {
            sanitized_text = text.replace('\n', " ").replace('\r', "");
            &sanitized_text
        };

        self.push_undo_patch(range.clone(), text_to_insert.len());

        if let Some(cached_len) = self.cached_utf16_len {
            let removed_utf16_len: usize = self.content[range.clone()]
                .chars()
                .map(|c| c.len_utf16())
                .sum();
            let added_utf16_len: usize = text_to_insert.chars().map(|c| c.len_utf16()).sum();
            self.cached_utf16_len = Some(cached_len - removed_utf16_len + added_utf16_len);
        }

        self.content.replace_range(range.clone(), text_to_insert);
        self.selected_range =
            range.start + text_to_insert.len()..range.start + text_to_insert.len();
        self.marked_range.take();
        self.needs_layout = true;
        self.pause_cursor_blink(cx);
        cx.emit(InputStateEvent::TextChanged);
        cx.notify();
    }

    pub(crate) fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.selected_range = 0..self.content.len();
        self.selection_reversed = false;
        cx.notify();
    }

    pub(crate) fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let new_pos = self.previous_boundary(self.cursor_offset());
            self.move_to(new_pos, cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    pub(crate) fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let new_pos = self.next_boundary(self.cursor_offset());
            self.move_to(new_pos, cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    pub(crate) fn up(&mut self, _: &Up, _window: &mut Window, cx: &mut Context<Self>) {
        self.pause_cursor_blink(cx);
        if !self.multiline {
            self.selected_range = 0..0;
            self.selection_reversed = false;
            self.scroll_to_cursor();
            cx.notify();
            return;
        }
        if let Some(new_offset) = self.move_vertically(self.cursor_offset(), -1) {
            self.selected_range = new_offset..new_offset;
            self.selection_reversed = false;
            self.scroll_to_cursor();
            cx.notify();
        }
    }

    pub(crate) fn down(&mut self, _: &Down, _window: &mut Window, cx: &mut Context<Self>) {
        self.pause_cursor_blink(cx);
        if !self.multiline {
            let end = self.content.len();
            self.selected_range = end..end;
            self.selection_reversed = false;
            self.scroll_to_cursor();
            cx.notify();
            return;
        }
        if let Some(new_offset) = self.move_vertically(self.cursor_offset(), 1) {
            self.selected_range = new_offset..new_offset;
            self.selection_reversed = false;
            self.scroll_to_cursor();
            cx.notify();
        }
    }

    pub(crate) fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    pub(crate) fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    pub(crate) fn select_up(&mut self, _: &SelectUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.pause_cursor_blink(cx);
        if !self.multiline {
            self.select_to(0, cx);
            return;
        }
        if let Some(new_offset) = self.move_vertically(self.cursor_offset(), -1) {
            if self.selection_reversed {
                self.selected_range.start = new_offset;
            } else {
                self.selected_range.end = new_offset;
            }
            if self.selected_range.end < self.selected_range.start {
                self.selection_reversed = !self.selection_reversed;
                self.selected_range = self.selected_range.end..self.selected_range.start;
            }
            self.scroll_to_cursor();
            cx.notify();
        }
    }

    pub(crate) fn select_down(
        &mut self,
        _: &SelectDown,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pause_cursor_blink(cx);
        if !self.multiline {
            self.select_to(self.content.len(), cx);
            return;
        }
        if let Some(new_offset) = self.move_vertically(self.cursor_offset(), 1) {
            if self.selection_reversed {
                self.selected_range.start = new_offset;
            } else {
                self.selected_range.end = new_offset;
            }
            if self.selected_range.end < self.selected_range.start {
                self.selection_reversed = !self.selection_reversed;
                self.selected_range = self.selected_range.end..self.selected_range.start;
            }
            self.scroll_to_cursor();
            cx.notify();
        }
    }

    pub(crate) fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        let line_start = self.find_line_start(self.cursor_offset());
        self.move_to(line_start, cx);
    }

    pub(crate) fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        let line_end = self.find_line_end(self.cursor_offset());
        self.move_to(line_end, cx);
    }

    pub(crate) fn move_to_beginning(
        &mut self,
        _: &MoveToBeginning,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_to(0, cx);
    }

    pub(crate) fn move_to_end(&mut self, _: &MoveToEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    pub(crate) fn select_to_beginning(
        &mut self,
        _: &SelectToBeginning,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(0, cx);
    }

    pub(crate) fn select_to_end(
        &mut self,
        _: &SelectToEnd,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.content.len(), cx);
    }

    pub(crate) fn word_left(&mut self, _: &WordLeft, _: &mut Window, cx: &mut Context<Self>) {
        let new_pos = self.previous_word_boundary(self.cursor_offset());
        self.move_to(new_pos, cx);
    }

    pub(crate) fn word_right(&mut self, _: &WordRight, _: &mut Window, cx: &mut Context<Self>) {
        let new_pos = self.next_word_boundary(self.cursor_offset());
        self.move_to(new_pos, cx);
    }

    pub(crate) fn select_word_left(
        &mut self,
        _: &SelectWordLeft,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_pos = self.previous_word_boundary(self.cursor_offset());
        self.select_to(new_pos, cx);
    }

    pub(crate) fn select_word_right(
        &mut self,
        _: &SelectWordRight,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_pos = self.next_word_boundary(self.cursor_offset());
        self.select_to(new_pos, cx);
    }

    pub(crate) fn enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        if self.multiline {
            self.replace_text_in_range(None, "\n", window, cx);
        }
    }

    pub(crate) fn tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        self.replace_text_in_range(None, "\t", window, cx);
    }

    pub(crate) fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn delete_word_left(
        &mut self,
        _: &DeleteWordLeft,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_word_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn delete_word_right(
        &mut self,
        _: &DeleteWordRight,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_word_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn delete_to_beginning_of_line(
        &mut self,
        _: &DeleteToBeginningOfLine,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            self.select_to(self.find_line_start(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn delete_to_end_of_line(
        &mut self,
        _: &DeleteToEndOfLine,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            self.select_to(self.find_line_end(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            if self.multiline {
                self.replace_text_in_range(None, &text, window, cx);
            } else {
                let text = text.replace('\n', " ").replace('\r', "");
                self.replace_text_in_range(None, &text, window, cx);
            }
        }
    }

    pub(crate) fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    pub(crate) fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx);
        } else {
            let cursor = self.cursor_offset();
            let line_start = self.find_line_start(cursor);
            let line_end = self.find_line_end(cursor);

            let cut_end = if line_end < self.content.len() {
                line_end + 1
            } else if line_start > 0 {
                line_end
            } else {
                line_end
            };

            let cut_start = if line_end >= self.content.len() && line_start > 0 {
                line_start - 1
            } else {
                line_start
            };

            let line_text = self.content[cut_start..cut_end].to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(line_text));

            self.selected_range = cut_start..cut_end;
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    pub(crate) fn on_mouse_down(
        &mut self,
        position: Point<Pixels>,
        click_count: usize,
        shift: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle);
        self.is_selecting = true;

        let is_same_position = self
            .last_click_position
            .map(|last| {
                let threshold = px(4.);
                (position.x - last.x).abs() < threshold && (position.y - last.y).abs() < threshold
            })
            .unwrap_or(false);

        if is_same_position && click_count > 1 {
            self.click_count = click_count;
        } else {
            self.click_count = 1;
        }
        self.last_click_position = Some(position);

        let clicked_offset = self.index_for_position(position);

        match self.click_count {
            2 => {
                let (word_start, word_end) = self.word_range_at(clicked_offset);
                self.selected_range = word_start..word_end;
                self.selection_reversed = false;
                cx.notify();
            }
            3 => {
                let line_start = self.find_line_start(clicked_offset);
                let line_end = self.find_line_end(clicked_offset);
                let line_end_with_newline = if line_end < self.content.len() {
                    line_end + 1
                } else {
                    line_end
                };
                self.selected_range = line_start..line_end_with_newline;
                self.selection_reversed = false;
                cx.notify();
            }
            _ => {
                if shift {
                    self.select_to(clicked_offset, cx);
                } else {
                    self.move_to(clicked_offset, cx);
                }
            }
        }
    }

    pub(crate) fn on_mouse_up(&mut self, _cx: &mut Context<Self>) {
        self.is_selecting = false;
    }

    pub(crate) fn on_mouse_move(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        if self.is_selecting && self.click_count == 1 {
            self.select_to(self.index_for_position(position), cx);
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.pause_cursor_blink(cx);
        let offset = offset.min(self.content.len());
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.scroll_to_cursor();
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.pause_cursor_blink(cx);
        let offset = offset.min(self.content.len());
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        self.scroll_to_cursor();
        cx.notify();
    }

    pub(crate) fn find_line_start(&self, offset: usize) -> usize {
        self.content[..offset.min(self.content.len())]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0)
    }

    pub(crate) fn find_line_end(&self, offset: usize) -> usize {
        self.content[offset.min(self.content.len())..]
            .find('\n')
            .map(|pos| offset + pos)
            .unwrap_or(self.content.len())
    }

    fn move_vertically(&self, offset: usize, direction: i32) -> Option<usize> {
        let (visual_line_idx, x_pixels) = self.find_visual_line_and_x_offset(offset);
        let target_visual_line_idx = (visual_line_idx as i32 + direction).max(0) as usize;

        let mut current_visual_line = 0;
        for layout in self.line_layouts.iter() {
            let visual_lines_in_layout = layout.visual_line_count;

            if target_visual_line_idx < current_visual_line + visual_lines_in_layout {
                let visual_line_within_layout = target_visual_line_idx - current_visual_line;

                if layout.text_range.is_empty() {
                    return Some(layout.text_range.start);
                }

                if let Some(wrapped) = &layout.wrapped_line {
                    let y_within_wrapped = self.line_height * visual_line_within_layout as f32;
                    let target_point = point(px(x_pixels), y_within_wrapped);

                    let closest_result =
                        wrapped.closest_index_for_position(target_point, self.line_height);

                    let closest_idx = closest_result.unwrap_or_else(|closest| closest);
                    let clamped = closest_idx.min(wrapped.text.len());
                    let result = layout.text_range.start + clamped;

                    return Some(result);
                }

                return Some(layout.text_range.start);
            }

            current_visual_line += visual_lines_in_layout;
        }

        if direction > 0 {
            Some(self.content.len())
        } else {
            None
        }
    }

    fn find_visual_line_and_x_offset(&self, offset: usize) -> (usize, f32) {
        if self.line_layouts.is_empty() {
            return (0, 0.0);
        }

        let mut visual_line_idx = 0;

        for line in &self.line_layouts {
            if line.text_range.is_empty() {
                if offset == line.text_range.start {
                    return (visual_line_idx, 0.0);
                }
            } else if offset >= line.text_range.start && offset <= line.text_range.end {
                if let Some(wrapped) = &line.wrapped_line {
                    let local_offset = (offset - line.text_range.start).min(wrapped.text.len());
                    if let Some(position) =
                        wrapped.position_for_index(local_offset, self.line_height)
                    {
                        let visual_line_within = (position.y / self.line_height).floor() as usize;
                        return (visual_line_idx + visual_line_within, position.x.into());
                    }
                }
                return (visual_line_idx, 0.0);
            }
            visual_line_idx += line.visual_line_count;
        }

        (visual_line_idx.saturating_sub(1), 0.0)
    }

    pub(crate) fn index_for_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }

        for line in self.line_layouts.iter() {
            let line_height_total = self.line_height * line.visual_line_count as f32;

            if position.y >= line.y_offset && position.y < line.y_offset + line_height_total {
                if line.text_range.is_empty() {
                    return line.text_range.start;
                }

                if let Some(wrapped) = &line.wrapped_line {
                    let relative_y = position.y - line.y_offset;
                    let relative_point = point(position.x, relative_y);

                    let closest_result =
                        wrapped.closest_index_for_position(relative_point, self.line_height);

                    let local_idx = closest_result.unwrap_or_else(|closest| closest);
                    let clamped = local_idx.min(wrapped.text.len());
                    return line.text_range.start + clamped;
                }
                return line.text_range.start;
            }
        }

        self.content.len()
    }

    pub(crate) fn scroll_to_cursor(&mut self) {
        if self.line_layouts.is_empty() {
            return;
        }

        let cursor_offset = if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        };

        if self.multiline {
            self.scroll_to_cursor_vertical(cursor_offset);
        } else {
            self.scroll_to_cursor_horizontal(cursor_offset);
        }
    }

    fn scroll_to_cursor_vertical(&mut self, cursor_offset: usize) {
        if self.available_height <= px(0.) {
            return;
        }

        let line_height = self.line_height;

        for line in &self.line_layouts {
            let is_cursor_in_line = if line.text_range.is_empty() {
                cursor_offset == line.text_range.start
            } else {
                line.text_range.contains(&cursor_offset)
                    || (cursor_offset == line.text_range.end && cursor_offset == self.content.len())
            };

            if is_cursor_in_line {
                let cursor_visual_y = if let Some(wrapped) = &line.wrapped_line {
                    let local_offset = cursor_offset.saturating_sub(line.text_range.start);
                    if let Some(position) =
                        wrapped.position_for_index(local_offset, self.line_height)
                    {
                        line.y_offset + position.y
                    } else {
                        line.y_offset
                    }
                } else {
                    line.y_offset
                };

                let visible_top = self.scroll_offset;
                let visible_bottom = self.scroll_offset + self.available_height;

                if cursor_visual_y < visible_top {
                    self.scroll_offset = cursor_visual_y;
                } else if cursor_visual_y + line_height > visible_bottom {
                    self.scroll_offset = (cursor_visual_y + line_height) - self.available_height;
                }

                self.scroll_offset = self.scroll_offset.max(px(0.));
                break;
            }
        }
    }

    fn scroll_to_cursor_horizontal(&mut self, cursor_offset: usize) {
        if self.available_width <= px(0.) {
            return;
        }

        let Some(line) = self.line_layouts.first() else {
            return;
        };

        let cursor_x = if let Some(wrapped) = &line.wrapped_line {
            let local_offset = cursor_offset.saturating_sub(line.text_range.start);
            wrapped
                .position_for_index(local_offset, self.line_height)
                .map(|p| p.x)
                .unwrap_or(px(0.))
        } else {
            px(0.)
        };

        let visible_left = self.scroll_offset;
        let visible_right = self.scroll_offset + self.available_width;
        let padding = px(2.0);

        if cursor_x < visible_left + padding {
            self.scroll_offset = (cursor_x - padding).max(px(0.));
        } else if cursor_x > visible_right - padding {
            self.scroll_offset = cursor_x - self.available_width + padding;
        }

        self.scroll_offset = self.scroll_offset.max(px(0.));
    }

    pub(crate) fn update_line_layouts(
        &mut self,
        width: Pixels,
        line_height: Pixels,
        text_style: &TextStyle,
        window: &mut Window,
    ) {
        self.line_height = line_height;
        self.set_text_style(text_style);

        if !self.needs_layout && self.wrap_width == Some(width) {
            return;
        }

        self.line_layouts.clear();
        self.wrap_width = Some(width);

        let text_color = text_style.color;
        let font_size = text_style.font_size.to_pixels(window.rem_size());

        if self.content.is_empty() {
            self.line_layouts.push(InputLineLayout {
                text_range: 0..0,
                wrapped_line: None,
                y_offset: px(0.),
                visual_line_count: 1,
                direction: TextDirection::default(),
            });
            self.needs_layout = false;
            return;
        }

        let mut last_direction = TextDirection::default();
        let mut y_offset = px(0.);
        let mut current_pos = 0;

        while current_pos < self.content.len() {
            let line_end = self.content[current_pos..]
                .find('\n')
                .map(|pos| current_pos + pos)
                .unwrap_or(self.content.len());

            let line_text = &self.content[current_pos..line_end];

            if line_text.is_empty() {
                self.line_layouts.push(InputLineLayout {
                    text_range: current_pos..current_pos,
                    wrapped_line: None,
                    y_offset,
                    visual_line_count: 1,
                    direction: last_direction,
                });
                y_offset += line_height;
            } else {
                let direction = detect_base_direction(line_text);
                last_direction = direction;
                let run = TextRun {
                    len: line_text.len(),
                    font: text_style.font(),
                    color: text_color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };

                let wrapped_lines = window
                    .text_system()
                    .shape_text(
                        SharedString::from(line_text.to_string()),
                        font_size,
                        &[run],
                        Some(width),
                        None,
                    )
                    .unwrap_or_default();

                for wrapped in wrapped_lines {
                    let visual_line_count = wrapped.wrap_boundaries().len() + 1;
                    let line_height_total = line_height * visual_line_count as f32;

                    self.line_layouts.push(InputLineLayout {
                        text_range: current_pos..line_end,
                        wrapped_line: Some(wrapped),
                        y_offset,
                        visual_line_count,
                        direction,
                    });

                    y_offset += line_height_total;
                }
            }

            current_pos = if line_end < self.content.len() {
                line_end + 1
            } else {
                self.content.len()
            };
        }

        if self.content.ends_with('\n') {
            self.line_layouts.push(InputLineLayout {
                text_range: self.content.len()..self.content.len(),
                wrapped_line: None,
                y_offset,
                visual_line_count: 1,
                direction: last_direction,
            });
        }

        self.needs_layout = false;
        self.scroll_to_cursor();
    }

    pub(crate) fn total_content_height(&self) -> Pixels {
        self.line_layouts
            .last()
            .map(|last| last.y_offset + self.line_height * last.visual_line_count as f32)
            .unwrap_or(px(0.))
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        if offset == 0 {
            return 0;
        }

        if let Some(utf16_len) = self.cached_utf16_len {
            if offset >= utf16_len {
                return self.content.len();
            }
        }

        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for character in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += character.len_utf16();
            utf8_offset += character.len_utf8();
        }

        utf8_offset.min(self.content.len())
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        if offset == 0 {
            return 0;
        }

        if offset >= self.content.len() {
            return self.utf16_len();
        }

        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for character in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += character.len_utf8();
            utf16_offset += character.len_utf16();
        }

        utf16_offset
    }

    fn utf16_len(&self) -> usize {
        if let Some(len) = self.cached_utf16_len {
            return len;
        }
        self.content.chars().map(|c| c.len_utf16()).sum()
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        if offset == 0 {
            return 0;
        }

        let text_before = &self.content[..offset.min(self.content.len())];
        text_before
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .next_back()
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        if offset >= self.content.len() {
            return self.content.len();
        }

        let text_after = &self.content[offset..];
        text_after
            .grapheme_indices(true)
            .nth(1)
            .map(|(i, _)| offset + i)
            .unwrap_or(self.content.len())
    }

    fn previous_word_boundary(&self, offset: usize) -> usize {
        if offset == 0 {
            return 0;
        }

        let text_before = &self.content[..offset.min(self.content.len())];

        let mut last_word_start = 0;
        for (idx, _) in text_before.unicode_word_indices() {
            if idx < offset {
                last_word_start = idx;
            }
        }

        if last_word_start == 0 && offset > 0 {
            let trimmed = text_before.trim_end();
            if trimmed.is_empty() {
                return 0;
            }
            for (idx, _) in trimmed.unicode_word_indices() {
                last_word_start = idx;
            }
        }

        last_word_start
    }

    fn next_word_boundary(&self, offset: usize) -> usize {
        if offset >= self.content.len() {
            return self.content.len();
        }

        let text_after = &self.content[offset..];

        for (idx, word) in text_after.unicode_word_indices() {
            let word_end = offset + idx + word.len();
            if word_end > offset {
                return word_end;
            }
        }

        self.content.len()
    }

    fn word_range_at(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.content.len());

        for (idx, word) in self.content.unicode_word_indices() {
            let word_end = idx + word.len();
            if offset >= idx && offset <= word_end {
                return (idx, word_end);
            }
        }

        (offset, offset)
    }
}

impl EntityInputHandler for InputState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        let clamped_range = range.start.min(self.content.len())..range.end.min(self.content.len());
        adjusted_range.replace(self.range_to_utf16(&clamped_range));
        Some(self.content[clamped_range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(&self, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range.as_ref().map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let range = range.start.min(self.content.len())..range.end.min(self.content.len());

        let sanitized_text;
        let text_to_insert = if self.multiline {
            new_text
        } else {
            sanitized_text = new_text.replace('\n', " ").replace('\r', "");
            &sanitized_text
        };

        self.push_undo_patch(range.clone(), text_to_insert.len());

        if let Some(cached_len) = self.cached_utf16_len {
            let removed_utf16_len: usize = self.content[range.clone()]
                .chars()
                .map(|c| c.len_utf16())
                .sum();
            let added_utf16_len: usize = text_to_insert.chars().map(|c| c.len_utf16()).sum();
            self.cached_utf16_len = Some(cached_len - removed_utf16_len + added_utf16_len);
        }

        self.content.replace_range(range.clone(), text_to_insert);
        self.selected_range =
            range.start + text_to_insert.len()..range.start + text_to_insert.len();
        self.marked_range.take();
        self.needs_layout = true;
        self.pause_cursor_blink(cx);
        cx.emit(InputStateEvent::TextChanged);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        let range = range.start.min(self.content.len())..range.end.min(self.content.len());

        let sanitized_text;
        let text_to_insert = if self.multiline {
            new_text
        } else {
            sanitized_text = new_text.replace('\n', " ").replace('\r', "");
            &sanitized_text
        };

        if let Some(cached_len) = self.cached_utf16_len {
            let removed_utf16_len: usize = self.content[range.clone()]
                .chars()
                .map(|c| c.len_utf16())
                .sum();
            let added_utf16_len: usize = text_to_insert.chars().map(|c| c.len_utf16()).sum();
            self.cached_utf16_len = Some(cached_len - removed_utf16_len + added_utf16_len);
        }

        self.content.replace_range(range.clone(), text_to_insert);

        if !text_to_insert.is_empty() {
            self.marked_range = Some(range.start..range.start + text_to_insert.len());
        } else {
            self.marked_range = None;
        }

        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.start)
            .unwrap_or_else(|| {
                range.start + text_to_insert.len()..range.start + text_to_insert.len()
            });

        self.needs_layout = true;
        cx.emit(InputStateEvent::TextChanged);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(&range_utf16);

        for line in &self.line_layouts {
            if line.text_range.is_empty() {
                if range.start == line.text_range.start {
                    return Some(Bounds::from_corners(
                        point(bounds.left(), bounds.top() + line.y_offset),
                        point(
                            bounds.left() + px(4.),
                            bounds.top() + line.y_offset + self.line_height,
                        ),
                    ));
                }
            } else if line.text_range.contains(&range.start) {
                if let Some(wrapped) = &line.wrapped_line {
                    let local_start = range.start - line.text_range.start;
                    let local_end = (range.end - line.text_range.start).min(wrapped.text.len());

                    let start_pos = wrapped
                        .position_for_index(local_start, self.line_height)
                        .unwrap_or(point(px(0.), px(0.)));
                    let end_pos = wrapped
                        .position_for_index(local_end, self.line_height)
                        .unwrap_or_else(|| {
                            let last_line_y = self.line_height * (line.visual_line_count - 1) as f32;
                            point(wrapped.width(), last_line_y)
                        });

                    let start_visual_line = (start_pos.y / self.line_height).floor() as usize;
                    let end_visual_line = (end_pos.y / self.line_height).floor() as usize;

                    if start_visual_line == end_visual_line {
                        return Some(Bounds::from_corners(
                            point(
                                bounds.left() + start_pos.x,
                                bounds.top() + line.y_offset + start_pos.y,
                            ),
                            point(
                                bounds.left() + end_pos.x,
                                bounds.top() + line.y_offset + start_pos.y + self.line_height,
                            ),
                        ));
                    } else {
                        return Some(Bounds::from_corners(
                            point(
                                bounds.left() + start_pos.x,
                                bounds.top() + line.y_offset + start_pos.y,
                            ),
                            point(
                                bounds.left() + wrapped.width(),
                                bounds.top() + line.y_offset + start_pos.y + self.line_height,
                            ),
                        ));
                    }
                }
            }
        }
        None
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let index = self.index_for_position(point);
        Some(self.offset_to_utf16(index))
    }
}

impl Focusable for InputState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
