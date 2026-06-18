// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use edit::helpers::*;
use edit::input::{Action, vk};
use edit::tui::*;
use stdext::arena_format;

use crate::localization::*;
use crate::settings::Settings;
use crate::state::*;

pub fn draw_menubar(ctx: &mut Context, state: &mut State) {
    ctx.menubar_begin();
    ctx.attr_background_rgba(state.menubar_color_bg);
    ctx.attr_foreground_rgba(state.menubar_color_fg);
    {
        let contains_focus = ctx.contains_focus();

        if ctx.menubar_menu_begin(loc(LocId::File), 'F') {
            draw_menu_file(ctx, state);
        }
        if !contains_focus && ctx.consume_shortcut(vk::F10) {
            ctx.steal_focus();
        }
        if state.documents.active().is_some() {
            if ctx.menubar_menu_begin(loc(LocId::Edit), 'E') {
                draw_menu_edit(ctx, state);
            }
            if ctx.menubar_menu_begin(loc(LocId::View), 'V') {
                draw_menu_view(ctx, state);
            }
        }
        if ctx.menubar_menu_begin(loc(LocId::Help), 'H') {
            draw_menu_help(ctx, state);
        }
    }
    ctx.menubar_end();
}

fn draw_menu_file(ctx: &mut Context, state: &mut State) {
    if ctx.menubar_menu_button(
        loc(LocId::FileNew),
        'N',
        ctx.keybinding_key_for(Action::FileNew).unwrap_or(vk::NULL),
    ) {
        draw_add_untitled_document(ctx, state);
    }
    if ctx.menubar_menu_button(
        loc(LocId::FileOpen),
        'O',
        ctx.keybinding_key_for(Action::FileOpen).unwrap_or(vk::NULL),
    ) {
        state.wants_file_picker = StateFilePicker::Open;
    }
    if state.documents.active().is_some() {
        if ctx.menubar_menu_button(
            loc(LocId::FileSave),
            'S',
            ctx.keybinding_key_for(Action::FileSave).unwrap_or(vk::NULL),
        ) {
            state.wants_save = true;
        }
        if ctx.menubar_menu_button(
            loc(LocId::FileSaveAs),
            'A',
            ctx.keybinding_key_for(Action::FileSaveAs).unwrap_or(vk::NULL),
        ) {
            state.wants_file_picker = StateFilePicker::SaveAs;
        }
    }
    #[allow(irrefutable_let_patterns)]
    if let path = Settings::borrow().path.as_path()
        && !path.as_os_str().is_empty()
        && ctx.menubar_menu_button(loc(LocId::FilePreferences), 'P', vk::NULL)
    {
        match state.documents.add_file_path(path) {
            Ok(doc) => {
                if let mut tb = doc.buffer.borrow_mut()
                    && tb.text_length() == 0
                {
                    Settings::bootstrap(&mut tb);
                }
            }
            Err(err) => error_log_add(ctx, state, err),
        }
    }
    #[allow(irrefutable_let_patterns)]
    if let path = Settings::borrow().keybindings_path.as_path()
        && !path.as_os_str().is_empty()
        && ctx.menubar_menu_button(loc(LocId::FileKeyBindings), 'K', vk::NULL)
    {
        match state.documents.add_file_path(path) {
            Ok(doc) => {
                if let mut tb = doc.buffer.borrow_mut()
                    && tb.text_length() == 0
                {
                    Settings::bootstrap_keybindings(&mut tb);
                }
            }
            Err(err) => error_log_add(ctx, state, err),
        }
    }
    if state.documents.active().is_some()
        && ctx.menubar_menu_button(
            loc(LocId::FileClose),
            'C',
            ctx.keybinding_key_for(Action::FileClose).unwrap_or(vk::NULL),
        )
    {
        state.wants_close = true;
    }
    if ctx.menubar_menu_button(
        loc(LocId::FileExit),
        'X',
        ctx.keybinding_key_for(Action::FileExit).unwrap_or(vk::NULL),
    ) {
        state.wants_exit = true;
    }
    ctx.menubar_menu_end();
}

fn draw_menu_edit(ctx: &mut Context, state: &mut State) {
    let doc = state.documents.active().unwrap();
    let mut tb = doc.buffer.borrow_mut();

    if ctx.menubar_menu_button(
        loc(LocId::EditUndo),
        'U',
        ctx.keybinding_key_for(Action::Undo).unwrap_or(vk::NULL),
    ) {
        tb.undo();
        ctx.needs_rerender();
    }
    if ctx.menubar_menu_button(
        loc(LocId::EditRedo),
        'R',
        ctx.keybinding_key_for(Action::Redo).unwrap_or(vk::NULL),
    ) {
        tb.redo();
        ctx.needs_rerender();
    }
    if ctx.menubar_menu_button(
        loc(LocId::EditCut),
        'T',
        ctx.keybinding_key_for(Action::Cut).unwrap_or(vk::NULL),
    ) {
        tb.cut(ctx.clipboard_mut());
        ctx.needs_rerender();
    }
    if ctx.menubar_menu_button(
        loc(LocId::EditCopy),
        'C',
        ctx.keybinding_key_for(Action::Copy).unwrap_or(vk::NULL),
    ) {
        tb.copy(ctx.clipboard_mut());
        ctx.needs_rerender();
    }
    if ctx.menubar_menu_button(
        loc(LocId::EditPaste),
        'P',
        ctx.keybinding_key_for(Action::Paste).unwrap_or(vk::NULL),
    ) {
        tb.paste(ctx.clipboard_ref(), false);
        ctx.needs_rerender();
    }
    if state.wants_search.kind != StateSearchKind::Disabled {
        if ctx.menubar_menu_button(
            loc(LocId::EditFind),
            'F',
            ctx.keybinding_key_for(Action::Find).unwrap_or(vk::NULL),
        ) {
            state.wants_search.kind = StateSearchKind::Search;
            state.wants_search.focus = true;
        }
        if ctx.menubar_menu_button(
            loc(LocId::EditReplace),
            'L',
            ctx.keybinding_key_for(Action::Replace).unwrap_or(vk::NULL),
        ) {
            state.wants_search.kind = StateSearchKind::Replace;
            state.wants_search.focus = true;
        }
    }
    if ctx.menubar_menu_button(
        loc(LocId::EditSelectAll),
        'A',
        ctx.keybinding_key_for(Action::SelectAll).unwrap_or(vk::NULL),
    ) {
        tb.select_all();
        ctx.needs_rerender();
    }
    ctx.menubar_menu_end();
}

fn draw_menu_view(ctx: &mut Context, state: &mut State) {
    if let Some(doc) = state.documents.active() {
        let mut tb = doc.buffer.borrow_mut();
        let word_wrap = tb.is_word_wrap_enabled();

        // All values on the statusbar are currently document specific.
        if ctx.menubar_menu_button(loc(LocId::ViewFocusStatusbar), 'S', vk::NULL) {
            state.wants_statusbar_focus = true;
        }
        if ctx.menubar_menu_button(
            loc(LocId::ViewGoToFile),
            'F',
            ctx.keybinding_key_for(Action::GoToFile).unwrap_or(vk::NULL),
        ) {
            state.wants_go_to_file = true;
        }
        if ctx.menubar_menu_button(
            loc(LocId::FileGoto),
            'G',
            ctx.keybinding_key_for(Action::GoToLine).unwrap_or(vk::NULL),
        ) {
            state.wants_goto = true;
        }
        if ctx.menubar_menu_checkbox(
            loc(LocId::ViewWordWrap),
            'W',
            ctx.keybinding_key_for(Action::ToggleWordWrap).unwrap_or(vk::NULL),
            word_wrap,
        ) {
            tb.set_word_wrap(!word_wrap);
            ctx.needs_rerender();
        }
    }

    ctx.menubar_menu_end();
}

fn draw_menu_help(ctx: &mut Context, state: &mut State) {
    if ctx.menubar_menu_button(loc(LocId::HelpAbout), 'A', vk::NULL) {
        state.wants_about = true;
    }
    ctx.menubar_menu_end();
}

pub fn draw_dialog_about(ctx: &mut Context, state: &mut State) {
    ctx.modal_begin("about", loc(LocId::AboutDialogTitle));
    {
        ctx.block_begin("content");
        ctx.inherit_focus();
        ctx.attr_padding(Rect::three(1, 2, 1));
        {
            ctx.label("description", "Microsoft Edit");
            ctx.attr_overflow(Overflow::TruncateTail);
            ctx.attr_position(Position::Center);

            ctx.label(
                "version",
                &arena_format!(
                    ctx.arena(),
                    "{}{}",
                    loc(LocId::AboutDialogVersion),
                    env!("CARGO_PKG_VERSION")
                ),
            );
            ctx.attr_overflow(Overflow::TruncateHead);
            ctx.attr_position(Position::Center);

            ctx.label("copyright", "Copyright (c) Microsoft Corporation");
            ctx.attr_overflow(Overflow::TruncateTail);
            ctx.attr_position(Position::Center);

            ctx.block_begin("choices");
            ctx.inherit_focus();
            ctx.attr_padding(Rect::three(1, 2, 0));
            ctx.attr_position(Position::Center);
            {
                if ctx.button("ok", loc(LocId::Ok), ButtonStyle::default()) {
                    state.wants_about = false;
                }
                ctx.inherit_focus();
            }
            ctx.block_end();
        }
        ctx.block_end();
    }
    if ctx.modal_end() {
        state.wants_about = false;
    }
}
