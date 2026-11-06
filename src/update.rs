// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use iced::Task;

use crate::data::{Message, RScashM};

impl RScashM {
    // Apply each UI event and return any async work it needs.
    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NameChanged(name) => {
                self.d_ui_name = name;
                Task::none()
            }
            Message::AmountChanged(amount) => {
                self.d_ui_amount = amount;
                Task::none()
            }
            Message::CurrencyChanged(currency) => {
                self.d_ui_currency = currency;
                Task::none()
            }
            Message::DateChanged(date) => {
                self.d_ui_date = date;
                Task::none()
            }
            Message::TypeChanged(transaction_type) => {
                self.d_ui_type = transaction_type;
                Task::none()
            }
            Message::TransmitData => {
                if self.is_encrypted
                    && (!self.encryption_unlocked || self.encryption_password.is_empty())
                {
                    self.status = if self.encryption_unlocked {
                        "Enter a password before editing encrypted data".to_string()
                    } else {
                        "Unlock the encrypted data file before editing it".to_string()
                    };
                } else {
                    self.add_transaction();
                }
                Task::none()
            }
            Message::RemoveData(index) => {
                // Remove the selected row and save the updated ledger.
                if self.is_encrypted
                    && (!self.encryption_unlocked || self.encryption_password.is_empty())
                {
                    self.status = if self.encryption_unlocked {
                        "Enter a password before editing encrypted data".to_string()
                    } else {
                        "Unlock the encrypted data file before editing it".to_string()
                    };
                } else {
                    self.clear_transaction(index);
                }
                Task::none()
            }
            Message::EncryptionPasswordChanged(password) => {
                self.encryption_password = password;
                Task::none()
            }
            Message::CheckEncryption => {
                self.check_encryption();
                Task::none()
            }
            Message::ApplyEncryption => {
                self.apply_encryption();
                Task::none()
            }
            Message::FileReadTick => {
                // Retry the startup read while the ledger is unavailable.
                self.poll_file_read();
                Task::none()
            }
            Message::BalanceCurrencyChanged(currency) => {
                self.balance_currency = currency;
                Task::none()
            }
            Message::ToggleSettingsPanel => {
                self.show_settings_panel = !self.show_settings_panel;
                Task::none()
            }
            Message::SelectDataFolder => {
                self.select_data_folder();
                Task::none()
            }
            Message::ChooseFile => {
                self.choose_file();
                Task::none()
            }
            Message::SetTheme(theme) => {
                self.set_app_theme(theme);
                Task::none()
            }
            Message::UpdateRgbChannel {
                target,
                channel,
                value,
            } => {
                self.update_rgb_channel(target, channel, value);
                Task::none()
            }
            Message::ChooseWallpaper => {
                let directory = self
                    .appearance
                    .wallpaper_path
                    .as_deref()
                    .and_then(std::path::Path::parent)
                    .map(std::path::Path::to_path_buf);
                Task::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp", "webp"]);
                        if let Some(directory) = directory {
                            dialog = dialog.set_directory(directory);
                        }
                        dialog
                            .pick_file()
                            .await
                            .map(|file| file.path().to_path_buf())
                    },
                    Message::WallpaperChosen,
                )
            }
            Message::WallpaperChosen(path) => {
                self.set_wallpaper(path);
                Task::none()
            }
            Message::ClearWallpaper => {
                self.clear_wallpaper();
                Task::none()
            }
            Message::RemoveAllData => {
                if self.is_encrypted
                    && (!self.encryption_unlocked || self.encryption_password.is_empty())
                {
                    self.status = if self.encryption_unlocked {
                        "Enter a password before editing encrypted data".to_string()
                    } else {
                        "Unlock the encrypted data file before editing it".to_string()
                    };
                } else {
                    self.clear_all_transactions();
                }
                Task::none()
            }
        }
    }
}
