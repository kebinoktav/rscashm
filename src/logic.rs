// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::data::{
    AppConfig, AppTheme, ColorChannel, ColorTarget, Currency, Data, FileReadLoop, RScashM,
    RgbColor, VData,
};
use zeroize::Zeroize;

const CONFIG_PATH: &str = "config.json";
static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn write_file_atomically(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing file name"))?;
    let (temporary_path, mut temporary_file) = loop {
        let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary_name = format!(
            ".{}.{}.{}.tmp",
            file_name.to_string_lossy(),
            std::process::id(),
            sequence
        );
        let candidate = parent.join(temporary_name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&candidate) {
            Ok(file) => break (candidate, file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };

    let write_result = temporary_file
        .write_all(contents)
        .and_then(|()| temporary_file.sync_all());
    drop(temporary_file);
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    if let Err(error) = fs::rename(&temporary_path, path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    fs::File::open(parent)?.sync_all()
}

// Add comma separators without changing the amount's sign.
pub(crate) fn int_to_text(amount: i128) -> String {
    let value = amount.to_string();
    let (sign, digits) = value
        .strip_prefix('-')
        .map_or(("", value.as_str()), |digits| ("-", digits));
    let mut result = String::with_capacity(value.len() + digits.len() / 3);
    result.push_str(sign);

    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

#[derive(Debug, PartialEq, Eq)]
// Why an amount input could not be read.
enum AmountError {
    // The input has something other than digits or spaces.
    Invalid,
    // The number is larger than i128 can hold.
    TooLarge,
}

// Read a whole-number amount, allowing spaces between digit groups.
fn parse_amount(input: &str) -> Result<i128, AmountError> {
    let digits: String = input
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(AmountError::Invalid);
    }

    digits.parse().map_err(|_| AmountError::TooLarge)
}

// Map current and older transaction labels to the two supported types.
pub(crate) fn normalize_transaction_type(transaction_type: &str) -> Option<&'static str> {
    match transaction_type {
        "" | "INCOME" => Some("INCOME"),
        "EXPENSE" | "OUTCOME" => Some("EXPENSE"),
        _ => None,
    }
}

// Save the active ledger path so the next launch can reuse it.
pub(crate) fn save_ledger_path(path: &Path) -> Result<(), String> {
    let path = path
        .canonicalize()
        .map_err(|error| format!("Couldn't resolve data file path: {error}"))?;
    let mut config = load_app_config();
    config.ledger_path = Some(path);
    write_app_config(&config)
}

fn load_app_config() -> AppConfig {
    let mut config = fs::read(CONFIG_PATH)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| {
            let ledger_path = fs::read_to_string("cfg.txt").ok().map(|saved_path| {
                PathBuf::from(
                    saved_path.trim_end_matches(|character| character == '\r' || character == '\n'),
                )
            });
            let theme = match fs::read_to_string("theme.txt")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str()
            {
                "light" => AppTheme::Light,
                _ => AppTheme::Dark,
            };
            let appearance = fs::read("appearance.json")
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .unwrap_or_default();
            AppConfig {
                ledger_path,
                theme,
                appearance,
            }
        });

    if config
        .appearance
        .wallpaper_path
        .as_ref()
        .is_some_and(|path| !path.is_file())
    {
        config.appearance.wallpaper_path = None;
    }
    config
}

fn write_app_config(config: &AppConfig) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(config)
        .map_err(|error| format!("Couldn't encode config.json: {error}"))?;
    write_file_atomically(Path::new(CONFIG_PATH), &bytes)
        .map_err(|error| format!("Couldn't save config.json: {error}"))
}

// Accept only whole numbers that fit in one RGB channel.
fn parse_rgb_value(value: &str) -> Option<u8> {
    value.parse::<u8>().ok()
}

// Show a formatted amount followed by its currency code.
pub(crate) fn to_currency(amount: i128, currency: Currency) -> String {
    format!("{} {currency:?}", int_to_text(amount))
}

impl VData {
    // Encode the ledger and write it to the selected file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let config = bincode::config::standard();
        let data = bincode::serde::encode_to_vec(self, config).map_err(std::io::Error::other)?;
        write_file_atomically(path, &data)
    }

    // Read a ledger file; blank files count as an empty ledger.
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
        if content.iter().all(|byte| byte.is_ascii_whitespace()) {
            return Ok(Self::default());
        }
        let config = bincode::config::standard();
        let (data, _) = bincode::serde::decode_from_slice(&content, config)
            .map_err(|e| format!("Failed to decode Bincode: {}", e))?;
        Ok(data)
    }

    fn file_is_encrypted(path: &Path) -> bool {
        if Self::load_from_file(path).is_ok() {
            return false;
        }
        fs::read(path)
            .map(|content| cryptainer::CryptData::from_bytes(&content).is_ok())
            .unwrap_or(false)
    }

    fn save_encrypted_to_file(&self, path: &Path, password: &str) -> Result<(), String> {
        let mut rng = rand::rng();
        let encrypted = self
            .encrypt(password, &mut rng)
            .map_err(|error| format!("Encryption failed: {error}"))?;
        write_file_atomically(path, &encrypted.to_bytes())
            .map_err(|error| format!("Save error: {error}"))
    }
}

impl RScashM {
    // Build app state from the saved ledger and appearance settings.
    pub(crate) fn new() -> Self {
        let mut config = load_app_config();
        let p = config
            .ledger_path
            .take()
            .filter(|path| path.exists())
            .unwrap_or_else(|| PathBuf::from("d_f.bin"));
        config.ledger_path = Some(p.clone());
        let app_theme = config.theme;
        let appearance = config.appearance;
        let _ = write_app_config(&AppConfig {
            ledger_path: config.ledger_path,
            theme: app_theme,
            appearance: appearance.clone(),
        });
        let background = appearance
            .background
            .unwrap_or_else(|| RgbColor::default_background(app_theme))
            .channels()
            .map(|channel| channel.to_string());
        let text = appearance
            .text
            .unwrap_or_else(|| RgbColor::default_text(app_theme))
            .channels()
            .map(|channel| channel.to_string());
        let (d, is_encrypted) = match VData::load_from_file(&p) {
            Ok(data) => (data, false),
            Err(_) if VData::file_is_encrypted(&p) => (VData::default(), true),
            Err(error) => {
                eprintln!("warning initial load: {}", error);
                (VData::default(), false)
            }
        };
        RScashM {
            data: d,
            path: p,
            status: String::new(),
            d_ui_amount: String::new(),
            d_ui_name: String::new(),
            d_ui_type: String::new(),
            d_ui_currency: String::new(),
            d_ui_date: String::new(),
            encryption_password: Default::default(),
            is_encrypted,
            encryption_unlocked: !is_encrypted,
            // Start checking for the ledger file after launch.
            read_state: if is_encrypted {
                FileReadLoop::Loaded(VData::default())
            } else {
                FileReadLoop::Waiting
            },
            // Keep the balance view on USD until another currency is picked.
            balance_currency: "USD".to_string(),
            show_settings_panel: false,
            app_theme,
            appearance,
            background_rgb_inputs: background,
            text_rgb_inputs: text,
        }
    }

    // Fill the transaction form with values from an existing entry.
    fn fill(&mut self, name: &str, amount: &str, currency: &str, ty: &str) {
        self.d_ui_name = name.to_string();
        self.d_ui_amount = amount.to_string();
        self.d_ui_currency = currency.to_string();
        self.d_ui_type = ty.to_string();
    }

    // Save the current ledger to its active file.
    pub(crate) fn save(&mut self) -> Result<(), String> {
        if self.is_encrypted && !self.encryption_unlocked {
            return Err("Unlock the encrypted data file before editing it".to_string());
        }
        if self.is_encrypted {
            if self.encryption_password.is_empty() {
                return Err("Enter a password before saving encrypted data".to_string());
            }
            return self
                .data
                .save_encrypted_to_file(&self.path, &self.encryption_password);
        }
        self.data
            .save_to_file(&self.path)
            .map_err(|e| format!("Save error: {}", e))
    }

    pub(crate) fn check_encryption(&mut self) {
        match VData::load_from_file(&self.path) {
            Ok(data) => {
                self.wipe_ledger_data();
                self.data = data.clone();
                self.is_encrypted = false;
                self.encryption_unlocked = true;
                self.encryption_password.zeroize();
                self.read_state = FileReadLoop::Loaded(data);
                self.status = "Data file is not encrypted".to_string();
            }
            Err(_) if VData::file_is_encrypted(&self.path) => {
                if !self.is_encrypted {
                    self.wipe_ledger_data();
                    self.encryption_unlocked = false;
                }
                self.is_encrypted = true;
                if self.encryption_unlocked {
                    self.read_state = FileReadLoop::Loaded(self.data.clone());
                } else {
                    self.read_state = FileReadLoop::Loaded(VData::default());
                }
                self.status = if self.encryption_unlocked {
                    "Data file is encrypted and unlocked".to_string()
                } else {
                    "Encrypted data detected. Enter its password and choose Unlock".to_string()
                };
            }
            Err(error) => self.status = format!("Couldn't identify data file: {error}"),
        }
    }

    pub(crate) fn apply_encryption(&mut self) {
        if self.encryption_password.is_empty() {
            self.status = "Enter a password first".to_string();
            return;
        }

        if self.is_encrypted && !self.encryption_unlocked {
            let content = match fs::read(&self.path) {
                Ok(content) => content,
                Err(error) => {
                    self.status = format!("Couldn't read encrypted data: {error}");
                    return;
                }
            };
            match VData::decrypt_bytes(&content, &self.encryption_password) {
                Ok(data) => {
                    self.data = data;
                    self.encryption_unlocked = true;
                    self.read_state = FileReadLoop::Loaded(self.data.clone());
                    self.status = "Encrypted data unlocked".to_string();
                }
                Err(error) => self.status = format!("Couldn't unlock data: {error}"),
            }
            return;
        }

        match self
            .data
            .save_encrypted_to_file(&self.path, &self.encryption_password)
        {
            Ok(()) => {
                self.is_encrypted = true;
                self.encryption_unlocked = true;
                self.status = "Data encrypted".to_string();
            }
            Err(error) => self.status = error,
        }
    }

    pub(crate) fn lock_encryption(&mut self) {
        if !self.is_encrypted || !self.encryption_unlocked {
            return;
        }
        self.wipe_ledger_data();
        self.encryption_password.zeroize();
        self.encryption_unlocked = false;
        self.read_state = FileReadLoop::Loaded(VData::default());
        self.status = "Encrypted data locked".to_string();
    }

    fn wipe_ledger_data(&mut self) {
        self.data.zeroize_contents();
        if let FileReadLoop::Loaded(data) = &mut self.read_state {
            data.zeroize_contents();
        }
    }

    // Apply a built-in palette and save the user's choice.
    pub(crate) fn set_app_theme(&mut self, theme: AppTheme) {
        self.app_theme = theme;
        self.appearance.background = None;
        self.appearance.text = None;
        self.background_rgb_inputs = RgbColor::default_background(theme)
            .channels()
            .map(|channel| channel.to_string());
        self.text_rgb_inputs = RgbColor::default_text(theme)
            .channels()
            .map(|channel| channel.to_string());
        self.status = match self.save_app_config() {
            Ok(()) => "Theme updated".to_string(),
            Err(error) => error,
        };
    }

    fn save_app_config(&self) -> Result<(), String> {
        write_app_config(&AppConfig {
            ledger_path: Some(self.path.clone()),
            theme: self.app_theme,
            appearance: self.appearance.clone(),
        })
    }

    // Update one RGB channel and keep the settings file in sync.
    pub(crate) fn update_rgb_channel(
        &mut self,
        target: ColorTarget,
        channel: ColorChannel,
        value: String,
    ) {
        let index = channel.index();
        match target {
            ColorTarget::Background => self.background_rgb_inputs[index] = value.clone(),
            ColorTarget::Text => self.text_rgb_inputs[index] = value.clone(),
        }

        let Some(value) = parse_rgb_value(&value) else {
            self.status = "RGB values must be whole numbers from 0 to 255".to_string();
            return;
        };

        let mut color = match target {
            ColorTarget::Background => self
                .appearance
                .background
                .unwrap_or_else(|| RgbColor::default_background(self.app_theme)),
            ColorTarget::Text => self
                .appearance
                .text
                .unwrap_or_else(|| RgbColor::default_text(self.app_theme)),
        };
        match channel {
            ColorChannel::Red => color.red = value,
            ColorChannel::Green => color.green = value,
            ColorChannel::Blue => color.blue = value,
        }
        match target {
            ColorTarget::Background => self.appearance.background = Some(color),
            ColorTarget::Text => self.appearance.text = Some(color),
        }

        self.status = match self.save_app_config() {
            Ok(()) => "Appearance updated".to_string(),
            Err(error) => error,
        };
    }

    // Check a wallpaper path before saving it for the next launch.
    pub(crate) fn set_wallpaper(&mut self, selected_path: Option<PathBuf>) {
        let Some(selected_path) = selected_path else {
            return;
        };
        let path = match selected_path.canonicalize() {
            Ok(path) if path.is_file() => path,
            Ok(_) => {
                self.status = "Selected wallpaper is not a file".to_string();
                return;
            }
            Err(error) => {
                self.status = format!("Couldn't resolve wallpaper path: {error}");
                return;
            }
        };
        let supported_extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                ["png", "jpg", "jpeg", "gif", "bmp", "webp"]
                    .iter()
                    .any(|supported| extension.eq_ignore_ascii_case(supported))
            });
        if !supported_extension {
            self.status = "Choose a PNG, JPEG, GIF, BMP, or WebP wallpaper".to_string();
            return;
        }

        self.appearance.wallpaper_path = Some(path);
        self.status = match self.save_app_config() {
            Ok(()) => "Wallpaper updated".to_string(),
            Err(error) => error,
        };
    }

    // Remove the wallpaper choice and save the updated settings.
    pub(crate) fn clear_wallpaper(&mut self) {
        self.appearance.wallpaper_path = None;
        self.status = match self.save_app_config() {
            Ok(()) => "Wallpaper cleared".to_string(),
            Err(error) => error,
        };
    }

    // Load the chosen ledger and keep its path for future launches.
    pub(crate) fn choose_file(&mut self) {
        let initial_folder = self
            .path
            .parent()
            .filter(|folder| folder.is_dir())
            .unwrap_or_else(|| Path::new("."));
        let Some(selected_path) = rfd::FileDialog::new()
            .set_directory(initial_folder)
            .pick_file()
        else {
            return;
        };

        let path = match selected_path.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                self.status = format!("Couldn't resolve selected file path: {error}");
                return;
            }
        };

        match VData::load_from_file(&path) {
            Ok(data) => {
                self.wipe_ledger_data();
                self.data = data.clone();
                self.path = path.clone();
                self.is_encrypted = false;
                self.encryption_unlocked = true;
                self.encryption_password.zeroize();
                self.read_state = FileReadLoop::Loaded(data);
                self.status = match save_ledger_path(&path) {
                    Ok(()) => "File selected".to_string(),
                    Err(error) => error,
                };
            }
            Err(_) if VData::file_is_encrypted(&path) => {
                self.wipe_ledger_data();
                self.encryption_password.zeroize();
                self.path = path.clone();
                self.data = VData::default();
                self.is_encrypted = true;
                self.encryption_unlocked = false;
                self.read_state = FileReadLoop::Loaded(VData::default());
                self.status = match save_ledger_path(&path) {
                    Ok(()) => {
                        "Encrypted data detected. Enter its password and choose Unlock".to_string()
                    }
                    Err(error) => error,
                };
            }
            Err(error) => {
                self.status = format!("Failed to read selected file: {error}");
            }
        }
    }

    // Use a folder's d_f.bin file, creating it from the current ledger if needed.
    pub(crate) fn select_data_folder(&mut self) {
        if self.is_encrypted && (!self.encryption_unlocked || self.encryption_password.is_empty()) {
            self.status =
                "Unlock encrypted data and enter its password before changing folders".to_string();
            return;
        }
        let initial_folder = self
            .path
            .parent()
            .filter(|folder| folder.is_dir())
            .unwrap_or_else(|| Path::new("."));
        let Some(folder) = rfd::FileDialog::new()
            .set_directory(initial_folder)
            .pick_folder()
        else {
            return;
        };

        let new_path = folder.join("d_f.bin");
        if new_path == self.path {
            return;
        }

        let new_data = if new_path.exists() {
            match VData::load_from_file(&new_path) {
                Ok(data) => data,
                Err(_) if VData::file_is_encrypted(&new_path) => {
                    self.status =
                        "The folder contains encrypted data; select that file to unlock it"
                            .to_string();
                    return;
                }
                Err(error) => {
                    self.status = format!("Couldn't load {}: {error}", new_path.display());
                    return;
                }
            }
        } else {
            let save_result = if self.is_encrypted {
                self.data
                    .save_encrypted_to_file(&new_path, &self.encryption_password)
            } else {
                self.data
                    .save_to_file(&new_path)
                    .map_err(|error| format!("Save error: {error}"))
            };
            if let Err(error) = save_result {
                self.status = format!("Couldn't save {}: {error}", new_path.display());
                return;
            }
            self.data.clone()
        };

        if let Err(error) = save_ledger_path(&new_path) {
            self.status = error;
            return;
        }

        self.wipe_ledger_data();
        self.path = new_path;
        self.data = new_data.clone();
        self.read_state = FileReadLoop::Loaded(new_data);
        self.status = "Data folder updated".to_string();
    }

    // Clear the ledger and save the empty file.
    pub(crate) fn clear_all_transactions(&mut self) {
        self.data.v_data.clear();
        if let Err(e) = self.save() {
            self.status = format!("Save error: {}", e);
        } else {
            self.status = "Transaction removed".to_string();
        }
    }

    // Remove one row, or report when its index is out of range.
    pub(crate) fn clear_transaction(&mut self, index: usize) {
        if index >= self.data.v_data.len() {
            self.status = "Index out of range".to_string();
            eprintln!(
                "Index out of range: {} (len = {})",
                index,
                self.data.v_data.len()
            );
            return;
        }

        self.data.v_data.remove(index);
        if let Err(e) = self.save() {
            self.status = format!("Save error: {}", e);
        } else {
            self.status = "Transaction removed".to_string();
        }
    }

    // Validate the form, add one transaction, and save the ledger.
    pub(crate) fn add_transaction(&mut self) {
        let name = self.d_ui_name.trim();
        let currency_str = self.d_ui_currency.trim();

        // Validate name first so the error shown is the most relevant one.
        if name.is_empty() {
            self.status = "Name cannot be empty".to_string();
            return;
        }

        let amount = match parse_amount(&self.d_ui_amount) {
            Ok(amount) => amount,
            Err(AmountError::Invalid) => {
                self.status = "Invalid amount".to_string();
                return;
            }
            Err(AmountError::TooLarge) => {
                self.status = "Amount is too large for i128".to_string();
                return;
            }
        };
        if amount == 0 {
            self.status = "Amount must be > 0".to_string();
            return;
        }

        let currency = currency_str.parse::<Currency>().unwrap_or(Currency::USD);
        let date = if self.d_ui_date.trim().is_empty() {
            chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()
        } else {
            self.d_ui_date.trim().to_string()
        };

        let transaction_type = normalize_transaction_type(&self.d_ui_type).unwrap_or("INCOME");
        let tx = Data {
            d_name: name.to_string(),
            d_type: transaction_type.to_string(),
            d_amount: amount,
            d_currency: currency,
            d_date: date,
        };

        self.data.v_data.push(tx);
        if let Err(e) = self.save() {
            self.status = format!("Save error: {}", e);
        } else {
            self.status = "Transaction added".to_string();
            self.d_ui_name.clear();
            self.d_ui_amount.clear();
            self.d_ui_currency.clear();
            self.d_ui_type = "INCOME".to_string();
            self.d_ui_date.clear();
        }
    }

    // Retry the file read until startup has loaded the ledger.
    pub(crate) fn poll_file_read(&mut self) {
        if matches!(self.read_state, FileReadLoop::Loaded(_)) {
            return;
        }

        match VData::load_from_file(&self.path) {
            Ok(data) => {
                self.data = data.clone();
                self.is_encrypted = false;
                self.encryption_unlocked = true;
                self.read_state = FileReadLoop::Loaded(data);
                self.status = "File loaded".to_string();
            }
            Err(_) if VData::file_is_encrypted(&self.path) => {
                self.is_encrypted = true;
                self.encryption_unlocked = false;
                self.read_state = FileReadLoop::Loaded(self.data.clone());
                self.status =
                    "Encrypted data detected. Enter its password and choose Unlock".to_string();
            }
            Err(e) => {
                self.status = format!("Waiting for file... ({e})");
                self.read_state = FileReadLoop::Error(e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AmountError, int_to_text, normalize_transaction_type, parse_amount, parse_rgb_value,
        to_currency, write_file_atomically,
    };
    use crate::data::Currency;
    use crate::data::{
        AppConfig, AppTheme, AppearanceSettings, Data, FileReadLoop, RScashM, RgbColor, VData,
    };
    use zeroize::Zeroizing;

    #[test]
    fn atomic_write_replaces_existing_file() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rscashm-atomic-{}-{unique_id}.bin",
            std::process::id()
        ));

        std::fs::write(&path, b"old ledger").unwrap();
        write_file_atomically(&path, b"new ledger").unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), b"new ledger");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn locking_clears_ledger_copies_and_password() {
        let ledger = VData {
            v_data: vec![Data {
                d_name: "private entry".to_string(),
                d_type: "INCOME".to_string(),
                d_amount: 42,
                d_currency: Currency::USD,
                d_date: "today".to_string(),
            }],
        };
        let mut state = RScashM {
            data: ledger.clone(),
            path: std::path::PathBuf::new(),
            status: String::new(),
            d_ui_amount: String::new(),
            d_ui_name: String::new(),
            d_ui_type: String::new(),
            d_ui_currency: String::new(),
            d_ui_date: String::new(),
            encryption_password: Zeroizing::new("secret".to_string()),
            is_encrypted: true,
            encryption_unlocked: true,
            read_state: FileReadLoop::Loaded(ledger),
            balance_currency: "USD".to_string(),
            show_settings_panel: false,
            app_theme: AppTheme::Dark,
            appearance: AppearanceSettings::default(),
            background_rgb_inputs: std::array::from_fn(|_| "0".to_string()),
            text_rgb_inputs: std::array::from_fn(|_| "255".to_string()),
        };

        state.lock_encryption();

        assert!(state.data.v_data.is_empty());
        assert!(state.encryption_password.is_empty());
        assert!(!state.encryption_unlocked);
        assert!(
            matches!(state.read_state, FileReadLoop::Loaded(ref data) if data.v_data.is_empty())
        );
    }

    #[test]
    fn app_config_round_trips_as_one_json_document() {
        let config = AppConfig {
            ledger_path: Some(std::path::PathBuf::from("ledger.bin")),
            theme: AppTheme::Light,
            appearance: AppearanceSettings {
                background: Some(RgbColor {
                    red: 12,
                    green: 34,
                    blue: 56,
                }),
                text: None,
                wallpaper_path: Some(std::path::PathBuf::from("wallpaper.png")),
            },
        };

        let encoded = serde_json::to_value(&config).unwrap();
        assert_eq!(encoded["theme"], "light");
        assert!(encoded.get("ledger_path").is_some());
        assert!(encoded.get("appearance").is_some());

        let decoded: AppConfig = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded.ledger_path, config.ledger_path);
        assert_eq!(decoded.theme, config.theme);
        assert_eq!(decoded.appearance.background, config.appearance.background);
        assert_eq!(decoded.appearance.text, config.appearance.text);
        assert_eq!(
            decoded.appearance.wallpaper_path,
            config.appearance.wallpaper_path
        );
    }

    // Check formatting with common currencies and the i128 minimum.
    #[test]
    fn formats_amount_with_currency_code() {
        assert_eq!(to_currency(1_234_567, Currency::USD), "1,234,567 USD");
        assert_eq!(to_currency(-1_234, Currency::IDR), "-1,234 IDR");
        assert_eq!(
            to_currency(i128::MIN, Currency::EUR),
            "-170,141,183,460,469,231,731,687,303,715,884,105,728 EUR"
        );
    }

    // Accept valid RGB bytes and reject values outside their range.
    #[test]
    fn validates_rgb_channels_as_u8_values() {
        assert_eq!(parse_rgb_value("0"), Some(0));
        assert_eq!(parse_rgb_value("255"), Some(255));
        assert_eq!(parse_rgb_value("256"), None);
        assert_eq!(parse_rgb_value("-1"), None);
        assert_eq!(parse_rgb_value(""), None);
    }

    // Make sure custom colors and wallpaper survive saving and loading.
    #[test]
    fn appearance_settings_round_trip_through_json() {
        let appearance = AppearanceSettings {
            background: Some(RgbColor {
                red: 12,
                green: 34,
                blue: 56,
            }),
            text: Some(RgbColor {
                red: 210,
                green: 220,
                blue: 230,
            }),
            wallpaper_path: Some(std::path::PathBuf::from("wallpaper.png")),
        };

        let encoded = serde_json::to_vec(&appearance).unwrap();
        let decoded: AppearanceSettings = serde_json::from_slice(&encoded).unwrap();

        assert_eq!(decoded.background, appearance.background);
        assert_eq!(decoded.text, appearance.text);
        assert_eq!(decoded.wallpaper_path, appearance.wallpaper_path);
    }

    // Treat empty and whitespace-only files as an empty ledger.
    #[test]
    fn reads_blank_files_as_empty_data() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        for (index, content) in [b"".as_slice(), b"\n", b" \t\r\n"].iter().enumerate() {
            let path = std::env::temp_dir().join(format!(
                "rscashm-empty-{}-{unique_id}-{index}.bin",
                std::process::id()
            ));

            std::fs::write(&path, content).unwrap();
            let result = VData::load_from_file(&path);
            std::fs::remove_file(&path).unwrap();

            assert!(result.unwrap().v_data.is_empty());
        }
    }

    #[test]
    fn encrypted_ledger_round_trips_and_rejects_wrong_password() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rscashm-encrypted-{}-{unique_id}.bin",
            std::process::id()
        ));
        let data = VData {
            v_data: vec![Data::default()],
        };

        data.save_encrypted_to_file(&path, "correct horse battery staple")
            .unwrap();
        assert!(VData::file_is_encrypted(&path));
        assert!(VData::load_from_file(&path).is_err());

        let content = std::fs::read(&path).unwrap();
        let decrypted = VData::decrypt_bytes(&content, "correct horse battery staple").unwrap();
        assert_eq!(decrypted.v_data.len(), 1);
        assert!(VData::decrypt_bytes(&content, "wrong password").is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn long_plaintext_ledger_is_not_detected_as_encrypted() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rscashm-plaintext-{}-{unique_id}.bin",
            std::process::id()
        ));
        let data = VData {
            v_data: vec![Data {
                d_name: "A long transaction name ".repeat(4),
                ..Data::default()
            }],
        };

        data.save_to_file(&path).unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() >= 44);
        assert!(!VData::file_is_encrypted(&path));
        std::fs::remove_file(path).unwrap();
    }

    // Parse grouped whole numbers without using floating-point values.
    #[test]
    fn parses_whitespace_grouped_i128_amounts_without_floats() {
        assert_eq!(parse_amount(" 1 234 567 "), Ok(1_234_567));
        assert_eq!(
            parse_amount("170 141 183 460 469 231 731 687 303 715 884 105 727"),
            Ok(i128::MAX)
        );
        assert_eq!(parse_amount("12.5"), Err(AmountError::Invalid));
        assert_eq!(parse_amount("1,000"), Err(AmountError::Invalid));
        assert_eq!(
            parse_amount("170141183460469231731687303715884105728"),
            Err(AmountError::TooLarge)
        );
    }

    // Keep current and legacy transaction labels working.
    #[test]
    fn normalizes_income_expense_and_legacy_outcome() {
        assert_eq!(normalize_transaction_type(""), Some("INCOME"));
        assert_eq!(normalize_transaction_type("INCOME"), Some("INCOME"));
        assert_eq!(normalize_transaction_type("EXPENSE"), Some("EXPENSE"));
        assert_eq!(normalize_transaction_type("OUTCOME"), Some("EXPENSE"));
        assert_eq!(normalize_transaction_type("UNKNOWN"), None);
    }

    // Check comma grouping for positive, negative, and boundary values.
    #[test]
    fn formats_signed_i128_values() {
        assert_eq!(int_to_text(0), "0");
        assert_eq!(int_to_text(1_234), "1,234");
        assert_eq!(int_to_text(-1_234), "-1,234");
        assert_eq!(int_to_text(-1_234_567), "-1,234,567");
        assert_eq!(
            int_to_text(i128::MAX),
            "170,141,183,460,469,231,731,687,303,715,884,105,727"
        );
        assert_eq!(
            int_to_text(i128::MIN),
            "-170,141,183,460,469,231,731,687,303,715,884,105,728"
        );
    }
}
