use super::Mode;
use super::KeyMap;
use super::Key;
use super::Command;
use super::View;
use super::KeyMapState;
use super::LogEntries;
use super::EventStatus;
use super::Direction;
use super::Response;
use super::utils;


/// Standard mode is Iota's default mode.
///
/// Standard mode uses non-vi-like keybindings.
/// Unlike Normal, Command and Visual modes which are all used together, Standard
/// mode is used on its own.
///
/// Standard mode allows Iota to be used in a non-modal way, similar to mainstream
/// editors like emacs or sublime.
pub struct StandardMode {
    keymap: KeyMap,
}

impl StandardMode {

    pub fn new() -> StandardMode {
        StandardMode {
            keymap: StandardMode::key_defaults(),
        }
    }

    fn key_defaults() -> KeyMap {
        let mut keymap = KeyMap::new();

        // Editor Commands
        keymap.bind_key(Key::Ctrl('q'), Command::ExitEditor);
        keymap.bind_key(Key::Ctrl('s'), Command::SaveBuffer);
        keymap.bind_keys(vec![Key::Ctrl('x'), Key::Ctrl('c')].as_slice(), Command::ExitEditor);
        keymap.bind_keys(vec![Key::Ctrl('x'), Key::Ctrl('s')].as_slice(), Command::SaveBuffer);

        // Navigation
        keymap.bind_key(Key::Up, Command::MoveCursor(Direction::Up));
        keymap.bind_key(Key::Down, Command::MoveCursor(Direction::Down));
        keymap.bind_key(Key::Left, Command::MoveCursor(Direction::Left));
        keymap.bind_key(Key::Right, Command::MoveCursor(Direction::Right));

        keymap.bind_key(Key::Ctrl('p'), Command::MoveCursor(Direction::Up));
        keymap.bind_key(Key::Ctrl('n'), Command::MoveCursor(Direction::Down));
        keymap.bind_key(Key::Ctrl('b'), Command::MoveCursor(Direction::Left));
        keymap.bind_key(Key::Ctrl('f'), Command::MoveCursor(Direction::Right));

        keymap.bind_key(Key::Ctrl('e'), Command::LineEnd);
        keymap.bind_key(Key::Ctrl('a'), Command::LineStart);

        // Editing
        keymap.bind_key(Key::Tab, Command::InsertTab);
        keymap.bind_key(Key::Enter, Command::InsertLine);
        keymap.bind_key(Key::Backspace, Command::Delete(Direction::Left));
        keymap.bind_key(Key::Ctrl('h'), Command::Delete(Direction::Left));
        keymap.bind_key(Key::Delete, Command::Delete(Direction::Right));
        keymap.bind_key(Key::Ctrl('d'), Command::Delete(Direction::Right));

        // History
        keymap.bind_key(Key::Ctrl('y'), Command::Redo);
        keymap.bind_key(Key::Ctrl('z'), Command::Undo);

        keymap
    }

    fn handle_command(&mut self, c: Command, view: &mut View, log: &mut LogEntries) -> Response {
        match c {
            // Editor Commands
            Command::ExitEditor      => return Response::Quit,
            Command::SaveBuffer      => utils::save_buffer(&view.buffer),

            // Navigation
            Command::MoveCursor(dir) => view.move_cursor(dir),
            Command::LineEnd         => view.move_cursor_to_line_end(),
            Command::LineStart       => view.move_cursor_to_line_start(),

            // Editing
            Command::Delete(dir)     => {
                let mut transaction = log.start(view.cursor_data);
                view.delete_char(&mut transaction, dir);
            },
            Command::InsertTab       => {
                let mut transaction = log.start(view.cursor_data);
                view.insert_tab(&mut transaction);
            },
            Command::InsertLine      => {
                let mut transaction = log.start(view.cursor_data);
                view.insert_line(&mut transaction);
            },
            Command::InsertChar(c)   => {
                let mut transaction = log.start(view.cursor_data);
                view.insert_char(&mut transaction, c);
            },
            Command::Redo => {
                if let Some(entry) = log.redo() {
                    view.replay(entry);
                }
            }
            Command::Undo            => {
                if let Some(entry) = log.undo() {
                    view.replay(entry);
                }
            }
        }
        Response::Continue
    }
}

impl Mode for StandardMode {
    fn handle_key_event(&mut self, key: Option<Key>, view: &mut View, log: &mut LogEntries) -> EventStatus {
        let key = match key {
            Some(k) => k,
            None => return EventStatus::NotHandled
        };

        // send key to the keymap
        match self.keymap.check_key(key) {
            KeyMapState::Match(command) => {
                return EventStatus::Handled(self.handle_command(command, view, log));
            },
            KeyMapState::Continue => {
                // keep going and wait for more keypresses
                return EventStatus::Handled(Response::Continue)
            },
            KeyMapState::None => {}  // do nothing and handle the key normally
        }

        // if the key is a character that is not part of a keybinding, insert into the buffer
        // otherwise, ignore it.
        if let Key::Char(c) = key {
            let mut transaction = log.start(view.cursor_data);
            view.insert_char(&mut transaction, c);
            EventStatus::Handled(Response::Continue)
        } else {
            EventStatus::NotHandled
        }

    }

}
