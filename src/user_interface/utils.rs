// Copyright (c) 2017 Decode Detroit
// Author: Patton Doyle
// Licence: GNU GPLv3
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! A module to create a macro and several functions that simplify the steps of
//! creating and updating the user interface.

// Import the relevant structures into the correct namespace
use super::super::system_interface::{
    DisplayControl, DisplayDebug, DisplayType, DisplayWith, FullStatus, Hidden, ItemDescription,
    ItemPair, LabelControl, LabelHidden, StatusDescription,
};

// Import GTK library
extern crate gtk;
use self::gtk::prelude::*;

/// A macro to make moving clones into closures more convenient
///
/// This macro allows the user to easily and quickly clone any items in a
/// closure that would normally have to be manually cloned. This makes the
/// functions for individual connections much more elegant.
///
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

/// A function to clean any text provided by the user.
//
/// This function cleans any user- or system- provided text (the built-in
/// function with GTK+ glib::markup_escape_texts is not available in the
/// gtk-rs library). This function also truncates the text at the provided
/// max length to prevent the notifications from overflowing the available
/// space. Additional text (beyond max length) will be pushed to the next
/// line with an indentation if indent is true.
///
pub fn clean_text(
    raw_text: &str,
    max_length: usize,
    newline: bool,
    indent: bool,
    clean: bool,
) -> String {
    // If simply truncating withtout a newline
    let mut clean_text = String::new();
    if !newline {
        // Just copy the raw chars to the clean_text
        for (i, character) in raw_text.chars().enumerate() {
            // Stop at the maximum length
            if i > max_length {
                break;
            }

            // Otherwise add it to the clean text
            clean_text.push(character);
        }

    // Otherwise, prevent splitting words whenever possible
    } else {
        // Divide the new text into sections of whitespace
        let mut lines = 1;
        for word in raw_text.split_whitespace() {
            // If the size of the word exceeds our allotment, truncate it
            let size = word.chars().count();
            let count = clean_text.chars().count();
            if size > max_length {
                // Truncate the string at the first viable location
                let mut index = (max_length * lines) - count;
                loop {
                    // Only cut at a valid character boundary
                    match word.get(..index) {
                        // Add the truncated word
                        Some(truncated) => {
                            clean_text.push_str(truncated);
                            clean_text.push_str("\n... ");

                            // Increment the line count before moving on
                            lines += 1;
                            break;
                        }

                        // Try again with a longer word
                        None => index += 1,
                    }
                }

            // If the end of the word would be beyond our allotment, add a newline
            } else if (size + count) > (max_length * lines) {
                // Add the newline, tab if requested, and the word
                if indent {
                    clean_text.push_str("\n\t\t");
                } else {
                    clean_text.push('\n');
                }
                clean_text.push_str(word);
                clean_text.push(' ');

                // Increment the line count
                lines += 1;

            // Otherwise, just add the word and a space
            } else {
                clean_text.push_str(word);
                clean_text.push(' ');
            }
        }
    }

    // Return now if not cleaning
    if !clean {
        return clean_text;
    }

    // Replace any of the offending characters
    let mut final_text = String::new();
    for character in clean_text.chars() {
        // Catch and replace dangerous characters
        let mut safe_character: &str = &format!("{}", character);
        match character {
            // Replace the ampersand
            '&' => safe_character = "&amp;",

            // Replace the less than
            '<' => safe_character = "&lt;",

            // Replace the greater than
            '>' => safe_character = "&gt;",

            // Replace null character with nothing
            '\0' => safe_character = "",

            // Pass along any other characters
            _ => (),
        }

        // Add it to the final text
        final_text.push_str(safe_character);
    }
    final_text
}

/// A helper function to properly decorate a label. The function
/// sets the markup for the existing label and returns the priority from
/// the DisplayType, if it exists.
///
/// This function assumes that the text has already been cleaned and sized.
///
pub fn decorate_label(
    label: &gtk::Label,
    text: &str,
    display: DisplayType,
    full_status: &FullStatus,
    font_size: u32,
    high_contrast: bool,
) -> Option<u32> {
    // Decorate based on the display type
    match display {
        // Match the display control variant
        DisplayControl {
            color,
            highlight,
            highlight_state,
            priority,
        } => {
            // If high contrast mode, just set the size and return the priority
            if high_contrast {
                label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
                return priority;
            }

            // Set the markup color, if specified
            if let Some((red, green, blue)) = color {
                label.set_markup(&format!(
                    "<span color='#{:02X}{:02X}{:02X}' size='{}'>{}</span>",
                    red, green, blue, font_size, text
                ));

            // Default to default text color
            } else {
                label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
            }

            // Set the highlight color, if specified
            if let Some((red, green, blue)) = highlight {
                // Check to see if the highlight state is specified
                if let Some((status_id, state_id)) = highlight_state {
                    // Find the corresponding detail
                    if let Some(&StatusDescription { ref current, .. }) = full_status.get(
                        &ItemPair::from_item(status_id, ItemDescription::new("", Hidden)),
                    ) {
                        // If the current id matches the state id
                        if state_id == current.get_id() {
                            // Set the label to the highlight color
                            label.set_markup(&format!(
                                "<span color='#{:02X}{:02X}{:02X}' size='{}'>{}</span>",
                                red, green, blue, font_size, text
                            ));
                        }
                    }
                }
            }

            // Return the priority
            return priority;
        }

        // Match the display with variant
        DisplayWith {
            color,
            highlight,
            highlight_state,
            priority,
            ..
        } => {
            // If high contrast mode, just set the size and return the priority
            if high_contrast {
                label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
                return priority;
            }

            // Set the markup color, if specified
            if let Some((red, green, blue)) = color {
                label.set_markup(&format!(
                    "<span color='#{:02X}{:02X}{:02X}' size='{}'>{}</span>",
                    red, green, blue, font_size, text
                ));

            // Default to default text color
            } else {
                label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
            }

            // Set the highlight color, if specified
            if let Some((red, green, blue)) = highlight {
                // Check to see if the highlight state is specified
                if let Some((status_id, state_id)) = highlight_state {
                    // Find the corresponding detail
                    if let Some(&StatusDescription { ref current, .. }) = full_status.get(
                        &ItemPair::from_item(status_id, ItemDescription::new("", Hidden)),
                    ) {
                        // If the current id matches the state id
                        if state_id == current.get_id() {
                            // Set the label to the highlight color
                            label.set_markup(&format!(
                                "<span color='#{:02X}{:02X}{:02X}' size='{}'>{}</span>",
                                red, green, blue, font_size, text
                            ));
                        }
                    }
                }
            }

            // Return the priority
            return priority;
        }

        // Match the display debug variant
        DisplayDebug {
            color,
            highlight,
            highlight_state,
            priority,
            ..
        } => {
            // If high contrast mode, just set the size and return the priority
            if high_contrast {
                label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
                return priority;
            }

            // Set the markup color, if specified
            if let Some((red, green, blue)) = color {
                label.set_markup(&format!(
                    "<span color='#{:02X}{:02X}{:02X}' size='{}'>{}</span>",
                    red, green, blue, font_size, text
                ));

            // Default to default text color
            } else {
                label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
            }

            // Set the highlight color, if specified
            if let Some((red, green, blue)) = highlight {
                // Check to see if the highlight state is specified
                if let Some((status_id, state_id)) = highlight_state {
                    // Find the corresponding detail
                    if let Some(&StatusDescription { ref current, .. }) = full_status.get(
                        &ItemPair::from_item(status_id, ItemDescription::new("", Hidden)),
                    ) {
                        // If the current id matches the state id
                        if state_id == current.get_id() {
                            // Set the label to the highlight color
                            label.set_markup(&format!(
                                "<span color='#{:02X}{:02X}{:02X}' size'{}'>{}</span>",
                                red, green, blue, font_size, text
                            ));
                        }
                    }
                }
            }

            // Return the priority
            return priority;
        }

        // Match the label control variant
        LabelControl {
            color,
            highlight,
            highlight_state,
            priority,
        } => {
            // If high contrast mode, just set the size and return the priority
            if high_contrast {
                label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
                return priority;
            }

            // Set the markup color, if specified
            if let Some((red, green, blue)) = color {
                label.set_markup(&format!(
                    "<span color='#{:02X}{:02X}{:02X}' size='{}'>{}</span>",
                    red, green, blue, font_size, text
                ));

            // Default to default text color
            } else {
                label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
            }

            // Set the highlight color, if specified
            if let Some((red, green, blue)) = highlight {
                // Check to see if the highlight state is specified
                if let Some((status_id, state_id)) = highlight_state {
                    // Find the corresponding detail
                    if let Some(&StatusDescription { ref current, .. }) = full_status.get(
                        &ItemPair::from_item(status_id, ItemDescription::new("", Hidden)),
                    ) {
                        // If the current id matches the state id
                        if state_id == current.get_id() {
                            // Set the label to the highlight color
                            label.set_markup(&format!(
                                "<span color='#{:02X}{:02X}{:02X}' size='{}'>{}</span>",
                                red, green, blue, font_size, text
                            ));
                        }
                    }
                }
            }

            // Return the priority
            return priority;
        }

        // Match the label hidden variant
        LabelHidden {
            color,
            highlight,
            highlight_state,
            priority,
        } => {
            // If high contrast mode, just set the size and return the priority
            if high_contrast {
                label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
                return priority;
            }

            // Set the markup color
            if let Some((red, green, blue)) = color {
                label.set_markup(&format!(
                    "<span color='#{:02X}{:02X}{:02X}' size='{}'>{}</span>",
                    red, green, blue, font_size, text
                ));

            // Default to default text color
            } else {
                label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
            }
            
            // Set the highlight color, if specified
            if let Some((red, green, blue)) = highlight {
                // Check to see if the highlight state is specified
                if let Some((status_id, state_id)) = highlight_state {
                    // Find the corresponding detail
                    if let Some(&StatusDescription { ref current, .. }) = full_status.get(
                        &ItemPair::from_item(status_id, ItemDescription::new("", Hidden)),
                    ) {
                        // If the current id matches the state id
                        if state_id == current.get_id() {
                            // Set the label to the highlight color
                            label.set_markup(&format!(
                                "<span color='#{:02X}{:02X}{:02X}' size='{}'>{}</span>",
                                red, green, blue, font_size, text
                            ));
                        }
                    }
                }
            }

            // Return the priority
            return priority;
        }

        // Otherwise, use the default color and priority
        Hidden => {
            label.set_markup(&format!("<span size='{}'>{}</span>", font_size, text));
            return None;
        }
    }
}
