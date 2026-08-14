// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// Data structures

/// Represents a single transaction entry.
/// - t_type: either "Income" or "Expense" (determines sign of amount)
/// - t_name: descriptive name for the transaction
/// - t_category: user-defined category (defaults to "General")
/// - t_amount: positive monetary value (sign handled by type)
/// - t_date: timestamp automatically set on creation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    t_type: String,
    t_name: String,
    t_category: String,
    t_amount: u128,
    t_date: String,
}

/// The main ledger holding a vector of transactions.
/// It provides methods to save and load itself from a binary file.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Ledger {
    book: Vec<Data>,
}

impl Ledger {
    /// Saves the ledger to a file at the given path using bincode serialization.
    /// The standard bincode configuration is used for encoding.
    pub fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let config = bincode::config::standard();
        let data = bincode::serde::encode_to_vec(self, config).map_err(std::io::Error::other)?;
        fs::write(path, data)
    }

    /// Loads a ledger from a binary file. Returns an error string if reading
    /// or decoding fails, so the caller can handle the failure gracefully.
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let config = bincode::config::standard();
        let (data, _) = bincode::serde::decode_from_slice(&content, config)
            .map_err(|e| format!("Failed to decode Bincode: {}", e))?;
        Ok(data)
    }
}

// Helper functions

/// Formats a u128 amount with thousand separators (e.g., "1,234,567").
/// This makes large numbers easier to read in the UI.
fn format_currency(amount: u128) -> String {
    let s = amount.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// Saves the currently active file path to a configuration file named "cfg.txt".
/// This allows the app to remember the last used ledger location.
fn save_cfg_path(path: &Path) {
    if let Err(e) = fs::write("cfg.txt", path.to_str().unwrap_or("")) {
        eprintln!("can't save config: {}", e);
    }
}

/// Reads the saved file path from "cfg.txt". Returns None if the file doesn't
/// exist or the path is invalid (e.g., no longer exists on disk).
fn load_config_path() -> Option<PathBuf> {
    fs::read_to_string("cfg.txt")
        .ok()
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| p.exists())
}

// Application state

/// Main application state for the cash manager.
/// It holds the ledger, the active file path, input fields, search query,
/// a confirmation flag for clearing all data, and a status message.
pub struct RsCashM {
    book: Ledger,                                    // All transactions
    file_path: PathBuf,                              // Current file used for save/load
    input_name: String,                              // Transaction name input
    input_amount: String,                            // Amount input (text)
    input_type: String,                              // "Income" or "Expense"
    input_category: String,                          // Category input
    search_query: String,                            // Filter text for transaction list
    confirm_clear: bool,                             // Toggle for clear confirmation
    status_message: Option<(String, egui::Color32)>, // (message, color)
}

impl Default for RsCashM {
    /// Creates a default application state by attempting to load the last
    /// used ledger. If the config file is missing or load fails, a new empty
    /// ledger is created and a default file name is used.
    fn default() -> Self {
        let path = load_config_path().unwrap_or_else(|| PathBuf::from("d_f.bin"));
        let ledger = match Ledger::load_from_file(&path) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("warning initial load: {}", e);
                Ledger::default()
            }
        };
        Self {
            book: ledger,
            file_path: path,
            input_name: String::new(),
            input_amount: String::new(),
            input_type: "Income".to_string(),
            input_category: String::new(),
            search_query: String::new(),
            confirm_clear: false,
            status_message: None,
        }
    }
}

impl RsCashM {
    /// Helper to save the current ledger to the active file path.
    /// Returns an error string if saving fails.
    fn save(&mut self) -> Result<(), String> {
        self.book
            .save_to_file(&self.file_path)
            .map_err(|e| format!("Save error: {}", e))
    }

    /// Adds a new transaction using the current input fields.
    /// Validates that name is not empty, amount contains only digits,
    /// and amount is greater than zero. If valid, creates a `Data` record,
    /// pushes it to the ledger, saves the file, and clears input fields.
    /// Sets a status message to inform the user of success or failure.
    fn add_transaction(&mut self) {
        // Filter amount input to only digits, to avoid panics from invalid input.
        let amount_str: String = self
            .input_amount
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();

        // Validation checks
        if self.input_name.trim().is_empty() {
            self.status_message = Some(("Name cannot be empty".to_string(), egui::Color32::RED));
            return;
        }
        if amount_str.is_empty() {
            self.status_message = Some(("Invalid amount".to_string(), egui::Color32::RED));
            return;
        }
        let amount = amount_str.parse::<u128>().unwrap_or(0);
        if amount == 0 {
            self.status_message = Some(("Amount must be > 0".to_string(), egui::Color32::RED));
            return;
        }

        // Build the transaction record
        let tx = Data {
            t_type: self.input_type.clone(),
            t_name: self.input_name.clone(),
            t_category: if self.input_category.trim().is_empty() {
                "General".to_string()
            } else {
                self.input_category.clone()
            },
            t_amount: amount,
            t_date: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        };

        self.book.book.push(tx);
        if let Err(e) = self.save() {
            self.status_message = Some((e, egui::Color32::RED));
        } else {
            self.status_message = Some(("Transaction added".to_string(), egui::Color32::GREEN));
            // Clear input fields after successful addition
            self.input_name.clear();
            self.input_amount.clear();
            self.input_category.clear();
        }
    }

    /// Clears all transactions after user confirmation (the `confirm_clear` flag).
    /// Empties the ledger, saves the empty state, and updates the status message.
    fn clear_all(&mut self) {
        self.book.book.clear();
        if let Err(e) = self.save() {
            self.status_message = Some((e, egui::Color32::RED));
        } else {
            self.status_message = Some((
                "All data cleared".to_string(),
                egui::Color32::from_rgb(200, 150, 50),
            ));
        }
        self.confirm_clear = false;
    }

    /// Removes a specific transaction by index and saves the updated ledger.
    fn delete_transaction(&mut self, index: usize) {
        self.book.book.remove(index);
        if let Err(e) = self.save() {
            self.status_message = Some((e, egui::Color32::RED));
        } else {
            self.status_message = Some((
                "Transaction deleted".to_string(),
                egui::Color32::from_rgb(200, 150, 50),
            ));
        }
    }

    /// Opens a folder picker dialog. Once a folder is selected, sets the
    /// active file path to "finance.bin" inside that folder, saves this path
    /// to the config file, and attempts to load the ledger from that file.
    /// If the file does not exist, a new empty ledger is started.
    fn load_from_selected_folder(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            let target = folder.join("finance.bin");
            self.file_path = target;
            save_cfg_path(&self.file_path);

            match Ledger::load_from_file(&self.file_path) {
                Ok(ledger) => {
                    self.book = ledger;
                    self.status_message = Some(("File Loaded".to_string(), egui::Color32::GREEN));
                }
                Err(e) => {
                    self.book = Ledger::default();
                    self.status_message = Some((
                        format!("New file: {}", e),
                        egui::Color32::from_rgb(200, 150, 0),
                    ));
                }
            }
        }
    }
    fn load_from_selected_file(&mut self) {
        if let Some(file) = rfd::FileDialog::new().pick_file() {
            match Ledger::load_from_file(&file) {
                Ok(ledger) => {
                    self.file_path = file;
                    self.book = ledger;
                    save_cfg_path(&self.file_path);
                    self.status_message = Some(("File Loaded".to_string(), egui::Color32::GREEN))
                }
                Err(e) => {
                    self.book = Ledger::default();
                    self.status_message = Some((
                        format!("New file: {}", e),
                        egui::Color32::from_rgb(200, 150, 0),
                    ));
                }
            }
        }
    }
}

// eframe::App implementation – UI rendering and event handling

impl eframe::App for RsCashM {
    /// The `update` method is called every frame and builds the entire UI.
    /// It defines the layout, handles user interactions, and updates the state.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply a custom dark theme: black backgrounds and white text for
        // non‑interactive elements. This improves readability on dark screens.
        let mut style = (*ctx.style()).clone();
        style.visuals.panel_fill = egui::Color32::BLACK;
        style.visuals.window_fill = egui::Color32::BLACK;
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::BLACK;
        // Ensure text for non‑interactive, inactive, hovered, and active
        // widgets is white (instead of the default grey).
        let white_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        style.visuals.widgets.noninteractive.fg_stroke = white_stroke;
        style.visuals.widgets.inactive.fg_stroke = white_stroke;
        style.visuals.widgets.hovered.fg_stroke = white_stroke;
        style.visuals.widgets.active.fg_stroke = white_stroke;
        ctx.set_style(style);

        // Top panel: title, clear button, and folder selector.
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Rust Simple Cash Manager");

                // Right‑aligned buttons: clear and folder selection.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // If confirmation is requested, show "Are you sure?" and two buttons.
                    if self.confirm_clear {
                        ui.colored_label(egui::Color32::RED, "Are you sure?");
                        if ui.button("Yes, clear").clicked() {
                            self.clear_all();
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_clear = false;
                        }
                    } else {
                        if ui.button("Clear All Data").clicked() {
                            self.confirm_clear = true; // Ask for confirmation next click
                        }
                    }

                    ui.separator();

                    if ui.button("Select Folder").clicked() {
                        self.load_from_selected_folder();
                    }

                    if ui.button("Select File").clicked() {
                        self.load_from_selected_file();
                    }
                });
            });
            // Display the currently active file path.
            ui.label(format!("Active file: {}", self.file_path.display()));
        });

        // Central panel contains everything else.
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show status message if present, with a close button.
            if let Some((msg, color)) = self.status_message.clone() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(msg).color(color).strong());
                    if ui.button("Close").clicked() {
                        self.status_message = None;
                    }
                });
                ui.separator();
            }

            // Calculate total income and expense from the ledger.
            let mut total_income = 0u128;
            let mut total_expense = 0u128;
            for d in &self.book.book {
                if d.t_type == "Income" {
                    total_income += d.t_amount;
                } else {
                    total_expense += d.t_amount;
                }
            }
            let net = total_income as i128 - total_expense as i128;

            // Financial summary panel: income, expense, net balance.
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.columns(3, |cols| {
                        // Total Income
                        cols[0].vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("Total Income").color(egui::Color32::YELLOW),
                            );
                            ui.label(
                                egui::RichText::new(format_currency(total_income))
                                    .color(egui::Color32::GREEN)
                                    .strong(),
                            );
                        });
                        // Total Expense
                        cols[1].vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("Total Expense").color(egui::Color32::YELLOW),
                            );
                            ui.label(
                                egui::RichText::new(format_currency(total_expense))
                                    .color(egui::Color32::RED)
                                    .strong(),
                            );
                        });
                        // Net Balance
                        cols[2].vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("Net Balance").color(egui::Color32::YELLOW),
                            );
                            let color = if net >= 0 {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::RED
                            };
                            ui.label(
                                egui::RichText::new(if net >= 0 {
                                    format_currency(net as u128)
                                } else {
                                    format!("-{}", format_currency((-net) as u128))
                                })
                                .color(color)
                                .size(18.0)
                                .strong(),
                            );
                        });
                    });
                });

            ui.add_space(10.0);

            // Input form for adding a new transaction.
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(11.0))
                .show(ui, |ui| {
                    ui.heading("Add Transaction");
                    ui.separator();

                    // Row 1: Type (combo box) and Category (text).
                    ui.columns(2, |cols| {
                        cols[0].horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt("type_combo")
                                .selected_text(&self.input_type)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.input_type,
                                        "Income".to_string(),
                                        "Income",
                                    );
                                    ui.selectable_value(
                                        &mut self.input_type,
                                        "Expense".to_string(),
                                        "Expense",
                                    );
                                });
                        });
                        cols[1].horizontal(|ui| {
                            ui.label("Category:");
                            ui.text_edit_singleline(&mut self.input_category);
                        });
                    });
                    // Row 2: Name and Amount.
                    ui.columns(2, |cols| {
                        cols[0].horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.input_name);
                        });
                        cols[1].horizontal(|ui| {
                            ui.label("Amount:");
                            ui.text_edit_singleline(&mut self.input_amount);
                        });
                    });

                    // Submit button; also allows pressing Enter to submit.
                    if ui.button("Submit").clicked() {
                        self.add_transaction();
                    } else if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.add_transaction();
                    }
                });

            ui.add_space(10.0);

            // Search bar and transaction history list.
            ui.horizontal(|ui| {
                ui.heading("Transaction History");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.search_query);
                });
            });
            ui.separator();

            // Scrollable list of transactions. Each entry shows name, category,
            // date, amount with sign, and a delete button.
            egui::ScrollArea::vertical().show(ui, |ui| {
                let search = self.search_query.to_lowercase();
                let mut to_delete = None;

                for (i, tx) in self.book.book.iter().enumerate() {
                    // Check if the transaction matches the search filter.
                    let matches = search.is_empty()
                        || tx.t_name.to_lowercase().contains(&search)
                        || tx.t_category.to_lowercase().contains(&search);

                    if matches {
                        egui::Frame::group(ui.style())
                            .inner_margin(egui::Margin::same(9.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Left side: name, category, date.
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(&tx.t_name).strong());
                                            ui.label(
                                                egui::RichText::new(format!("[{}]", tx.t_category))
                                                    .small()
                                                    .color(egui::Color32::LIGHT_BLUE),
                                            );
                                        });
                                        // Date displayed in white.
                                        ui.label(
                                            egui::RichText::new(&tx.t_date)
                                                .small()
                                                .color(egui::Color32::WHITE),
                                        );
                                    });

                                    // Right side: amount with sign and delete button.
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("Delete").clicked() {
                                                to_delete = Some(i);
                                            }
                                            let color = if tx.t_type == "Income" {
                                                egui::Color32::GREEN
                                            } else {
                                                egui::Color32::RED
                                            };
                                            let sign =
                                                if tx.t_type == "Income" { "+" } else { "-" };
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{} {}",
                                                    sign,
                                                    format_currency(tx.t_amount)
                                                ))
                                                .color(color)
                                                .strong(),
                                            );
                                        },
                                    );
                                });
                            });
                        ui.add_space(4.0);
                    }
                }

                // If a delete was requested, perform it after the loop to avoid
                // borrow conflicts with the iterator.
                if let Some(idx) = to_delete {
                    self.delete_transaction(idx);
                }
            });
        });
    }
}
