// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::data::{AppTheme, ColorChannel, ColorTarget, Currency, Message, RScashM, RgbColor};
use crate::logic::{int_to_text, normalize_transaction_type, to_currency};
use iced::{
    Alignment, Border, Color, ContentFit, Element, Length, Shadow, Theme,
    widget::{
        Space, button, column, container, image, pick_list, row, scrollable, text, text_input,
    },
};
use zeroize::Zeroizing;

// Share one currency list between the form and balance mode.
pub const CURRENCIES: &[&str] = &[
    "AED", "AFN", "ALL", "AMD", "AOA", "ARS", "AUD", "AWG", "AZN", "BAM", "BBD", "BDT", "BHD",
    "BIF", "BMD", "BND", "BOB", "BRL", "BSD", "BTN", "BWP", "BYN", "BZD", "CAD", "CDF", "CHF",
    "CLP", "CNY", "COP", "CRC", "CUP", "CVE", "CZK", "DJF", "DKK", "DOP", "DZD", "EGP", "ERN",
    "ETB", "EUR", "FJD", "FKP", "GBP", "GEL", "GHS", "GIP", "GMD", "GNF", "GTQ", "GYD", "HKD",
    "HNL", "HTG", "HUF", "IDR", "ILS", "INR", "IQD", "IRR", "ISK", "JMD", "JOD", "JPY", "KES",
    "KGS", "KHR", "KMF", "KPW", "KRW", "KWD", "KYD", "KZT", "LAK", "LBP", "LKR", "LRD", "LSL",
    "LYD", "MAD", "MDL", "MGA", "MKD", "MMK", "MNT", "MOP", "MRU", "MUR", "MVR", "MWK", "MXN",
    "MYR", "MZN", "NAD", "NGN", "NIO", "NOK", "NPR", "NZD", "OMR", "PAB", "PEN", "PGK", "PHP",
    "PKR", "PLN", "PYG", "QAR", "RON", "RSD", "RUB", "RWF", "SAR", "SBD", "SCR", "SDG", "SEK",
    "SGD", "SHP", "SLE", "SOS", "SRD", "SSP", "STN", "SVC", "SYP", "SZL", "THB", "TJS", "TMT",
    "TND", "TOP", "TRY", "TTD", "TWD", "TZS", "UAH", "UGX", "USD", "UYU", "UZS", "VED", "VES",
    "VND", "VUV", "WST", "XAF", "XCD", "XCG", "XOF", "XPF", "YER", "ZAR", "ZMW", "ZWG",
];
// Keep transaction choices to the two supported types.
const TRANSACTION_TYPES: &[&str] = &["INCOME", "EXPENSE"];
// Give the main pane most of the window when Settings is open.
const MAIN_PANE_PORTION: u16 = 3;
// Keep the Settings pane compact beside the main view.
const SETTINGS_PANE_PORTION: u16 = 1;

// Measure perceived brightness for an sRGB color.
fn relative_luminance(color: Color) -> f32 {
    // Turn sRGB channels into linear light so contrast math makes sense.
    let linear = |channel: f32| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

// Compare two colors using their relative brightness.
fn contrast_ratio(first: Color, second: Color) -> f32 {
    // Compare two colors by brightness and return the contrast ratio.
    let first_luminance = relative_luminance(first);
    // Keep the second luminance around for the same contrast check.
    let second_luminance = relative_luminance(second);
    // Contrast always puts the brighter value over the darker one.
    let (lighter, darker) = if first_luminance >= second_luminance {
        (first_luminance, second_luminance)
    } else {
        (second_luminance, first_luminance)
    };
    (lighter + 0.05) / (darker + 0.05)
}

// Keep text readable when its chosen color blends into the background.
fn readable_text_color(preferred: Color, background: Color) -> Color {
    // Pick a text color that won't disappear on the background.
    if contrast_ratio(preferred, background) >= 4.5 {
        return preferred;
    }

    if contrast_ratio(Color::WHITE, background) >= contrast_ratio(Color::BLACK, background) {
        Color::WHITE
    } else {
        Color::BLACK
    }
}

// Colors shared by the app's panels, text, rows, and inputs.
#[derive(Clone, Copy)]
struct UiColors {
    // background: page base color.
    background: Color,
    // panel: side panel / card background.
    panel: Color,
    // text: main readable text color.
    text: Color,
    // muted: softer text for hints and small labels.
    muted: Color,
    // heading: accent color for section headings.
    heading: Color,
    // input: text input background.
    input: Color,
    // input_border: text input border color.
    input_border: Color,
    // row: transaction row background.
    row: Color,
    // row_border: transaction row border color.
    row_border: Color,
    // divider: thin line separator color.
    divider: Color,
}

impl UiColors {
    // Combine a built-in palette with custom colors when they are set.
    fn for_theme(
        theme: AppTheme,
        custom_background: Option<RgbColor>,
        custom_text: Option<RgbColor>,
    ) -> Self {
        // Start with the selected built-in palette.
        let mut colors = match theme {
            AppTheme::Dark => Self {
                background: Color::BLACK,
                panel: Color::from_rgb(0.05, 0.05, 0.05),
                text: Color::WHITE,
                muted: Color::from_rgb(0.68, 0.68, 0.68),
                heading: Color::from_rgb(0.9, 0.8, 0.2),
                input: Color::from_rgb(0.10, 0.10, 0.10),
                input_border: Color::from_rgb(0.32, 0.32, 0.32),
                row: Color::from_rgb(0.08, 0.08, 0.08),
                row_border: Color::from_rgb(0.25, 0.25, 0.25),
                divider: Color::from_rgb(0.3, 0.3, 0.3),
            },
            AppTheme::Light => Self {
                background: Color::from_rgb(0.96, 0.97, 0.95),
                panel: Color::from_rgb(0.91, 0.93, 0.90),
                text: Color::from_rgb(0.10, 0.13, 0.12),
                muted: Color::from_rgb(0.35, 0.40, 0.38),
                heading: Color::from_rgb(0.35, 0.25, 0.08),
                input: Color::WHITE,
                input_border: Color::from_rgb(0.65, 0.69, 0.66),
                row: Color::WHITE,
                row_border: Color::from_rgb(0.78, 0.81, 0.78),
                divider: Color::from_rgb(0.72, 0.76, 0.73),
            },
        };
        // Use the custom page background when one is saved.
        if let Some(color) = custom_background {
            colors.background = Color::from_rgb8(color.red, color.green, color.blue);
        }
        // Use the custom text color before checking its contrast.
        if let Some(color) = custom_text {
            // Convert the saved RGB channels into an iced color.
            let preferred_text = Color::from_rgb8(color.red, color.green, color.blue);
            colors.text = preferred_text;
            colors.heading = preferred_text;
        }
        // Keep both text and headings readable on the final background.
        colors.text = readable_text_color(colors.text, colors.background);
        colors.heading = readable_text_color(colors.heading, colors.background);
        colors
    }
}

impl RScashM {
    // Return the main app screen.
    pub(crate) fn view<'a>(&'a self) -> Element<'a, Message> {
        self.main_view()
    }

    // Keep controls fixed while the transaction history scrolls.
    pub(crate) fn main_view<'a>(&'a self) -> Element<'a, Message> {
        // Apply the active palette to the page and wallpaper fallback.
        let colors = UiColors::for_theme(
            self.app_theme,
            self.appearance.background,
            self.appearance.text,
        );
        // Leave room for Settings while keeping the main pane wider.
        let main_width = if self.show_settings_panel {
            Length::FillPortion(MAIN_PANE_PORTION)
        } else {
            Length::Fill
        };
        // Check that the selected wallpaper still exists.
        let has_wallpaper = self
            .appearance
            .wallpaper_path
            .as_ref()
            .is_some_and(|path| path.is_file());
        // Keep the page controls and history inside a styled pane.
        let main_pane = container(self.page_content())
            .width(main_width)
            .height(Length::Fill)
            .padding(24)
            .style(move |_theme: &Theme| container::Style {
                background: if has_wallpaper {
                    None
                } else {
                    Some(colors.background.into())
                },
                text_color: Some(colors.text),
                ..container::Style::default()
            });
        // Place a selected wallpaper behind the main pane.
        let main_pane: Element<'a, Message> = if let Some(path) = self
            .appearance
            .wallpaper_path
            .as_ref()
            .filter(|path| path.is_file())
        {
            iced::widget::stack![
                image::Image::new(image::Handle::from_path(path))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(ContentFit::Cover)
                    .opacity(0.55_f32),
                main_pane,
            ]
            .width(main_width)
            .height(Length::Fill)
            .into()
        } else {
            main_pane.into()
        };

        // Show Settings beside the main pane when the panel is open.
        if self.show_settings_panel {
            row![main_pane, self.settings_panel_view()]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            main_pane.into()
        }
    }

    // Show theme, color, wallpaper, and ledger controls.
    fn settings_panel_view<'a>(&'a self) -> Element<'a, Message> {
        // Use the active palette for every Settings control.
        let colors = UiColors::for_theme(
            self.app_theme,
            self.appearance.background,
            self.appearance.text,
        );
        // Show the active ledger path in Settings.
        let data_file_path = self.path.display().to_string();
        // Show a clear action only when a wallpaper is selected.
        let wallpaper_control: Element<'a, Message> = if self.appearance.wallpaper_path.is_some() {
            button(text("Clear wallpaper").color(colors.text))
                .on_press(Message::ClearWallpaper)
                .padding([8, 12])
                .into()
        } else {
            text("No wallpaper selected")
                .size(12)
                .color(colors.muted)
                .into()
        };
        // Match the RGB inputs to the Settings panel colors.
        let rgb_input_style = move |_theme: &Theme, _status| text_input::Style {
            background: colors.input.into(),
            border: Border {
                color: colors.input_border,
                width: 1.0,
                radius: 4.0.into(),
            },
            icon: colors.text,
            placeholder: colors.muted,
            value: colors.text,
            selection: Color::from_rgb(0.25, 0.38, 0.52),
        };

        // Settings lists theme controls first, then the active ledger and file actions.
        container(scrollable(
            column![
                text("Settings").size(20).color(colors.text),
                container(Space::new().height(Length::Fixed(1.0)))
                    .width(Length::Fill)
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(colors.divider.into()),
                        ..container::Style::default()
                    }),
                text("Appearance").size(14).color(colors.heading),
                text(match self.app_theme {
                    AppTheme::Dark => "Dark mode is on",
                    AppTheme::Light => "White mode is on",
                })
                .size(12)
                .color(colors.muted),
                row![
                    button(text("Dark").color(colors.text))
                        .on_press(Message::SetTheme(AppTheme::Dark))
                        .padding([8, 12]),
                    button(text("White").color(colors.text))
                        .on_press(Message::SetTheme(AppTheme::Light))
                        .padding([8, 12]),
                ]
                .spacing(8),
                text("Background RGB (0-255)").size(12).color(colors.text),
                row![
                    text("R").width(Length::Fixed(20.0)),
                    text_input("0-255", &self.background_rgb_inputs[0])
                        .on_input(|value| Message::UpdateRgbChannel {
                            target: ColorTarget::Background,
                            channel: ColorChannel::Red,
                            value,
                        })
                        .width(Length::Fill)
                        .style(rgb_input_style),
                ]
                .spacing(6),
                row![
                    text("G").width(Length::Fixed(20.0)),
                    text_input("0-255", &self.background_rgb_inputs[1])
                        .on_input(|value| Message::UpdateRgbChannel {
                            target: ColorTarget::Background,
                            channel: ColorChannel::Green,
                            value,
                        })
                        .width(Length::Fill)
                        .style(rgb_input_style),
                ]
                .spacing(6),
                row![
                    text("B").width(Length::Fixed(20.0)),
                    text_input("0-255", &self.background_rgb_inputs[2])
                        .on_input(|value| Message::UpdateRgbChannel {
                            target: ColorTarget::Background,
                            channel: ColorChannel::Blue,
                            value,
                        })
                        .width(Length::Fill)
                        .style(rgb_input_style),
                ]
                .spacing(6),
                text("Text RGB (0-255)").size(12).color(colors.text),
                row![
                    text("R").width(Length::Fixed(20.0)),
                    text_input("0-255", &self.text_rgb_inputs[0])
                        .on_input(|value| Message::UpdateRgbChannel {
                            target: ColorTarget::Text,
                            channel: ColorChannel::Red,
                            value,
                        })
                        .width(Length::Fill)
                        .style(rgb_input_style),
                ]
                .spacing(6),
                row![
                    text("G").width(Length::Fixed(20.0)),
                    text_input("0-255", &self.text_rgb_inputs[1])
                        .on_input(|value| Message::UpdateRgbChannel {
                            target: ColorTarget::Text,
                            channel: ColorChannel::Green,
                            value,
                        })
                        .width(Length::Fill)
                        .style(rgb_input_style),
                ]
                .spacing(6),
                row![
                    text("B").width(Length::Fixed(20.0)),
                    text_input("0-255", &self.text_rgb_inputs[2])
                        .on_input(|value| Message::UpdateRgbChannel {
                            target: ColorTarget::Text,
                            channel: ColorChannel::Blue,
                            value,
                        })
                        .width(Length::Fill)
                        .style(rgb_input_style),
                ]
                .spacing(6),
                button(text("Choose wallpaper...").color(colors.text))
                    .on_press(Message::ChooseWallpaper)
                    .padding([8, 12]),
                wallpaper_control,
                container(Space::new().height(Length::Fixed(1.0)))
                    .width(Length::Fill)
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(colors.divider.into()),
                        ..container::Style::default()
                    }),
                text("Data").size(14).color(colors.heading),
                text("Current file").size(12).color(colors.text),
                text(data_file_path)
                    .size(12)
                    .color(colors.muted),
                button(text("Choose data folder...").color(colors.text))
                    .on_press(Message::SelectDataFolder)
                    .padding([8, 12]),
                button(text("Choose file...").color(colors.text))
                    .on_press(Message::ChooseFile)
                    .padding([8, 12]),
                text("The selected file keeps the same bincode data format. Its path is saved for next time.")
                    .size(12)
                    .color(colors.muted),
            ]
            .spacing(10)
            .width(Length::Fill),
        ))
        .width(Length::FillPortion(SETTINGS_PANE_PORTION))
        .height(Length::Fill)
        .padding(20)
        .style(move |_theme: &Theme| container::Style {
            background: Some(colors.panel.into()),
            text_color: Some(colors.text),
            border: Border {
                color: colors.divider,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
    }

    // Build the header, balance, transaction form, and history.
    fn page_content<'a>(&'a self) -> Element<'a, Message> {
        // Apply the active palette to the form, summary, and history.
        let colors = UiColors::for_theme(
            self.app_theme,
            self.appearance.background,
            self.appearance.text,
        );
        // Keep text fields readable against the selected background.
        let input_style = move |_theme: &Theme, _status| text_input::Style {
            background: colors.input.into(),
            border: Border {
                color: colors.input_border,
                width: 1.0,
                radius: 4.0.into(),
            },
            icon: colors.text,
            placeholder: colors.muted,
            value: colors.text,
            selection: Color::from_rgb(0.25, 0.38, 0.52),
        };

        // Use USD until a transaction currency has been selected.
        let selected_currency = if self.d_ui_currency.is_empty() {
            Some("USD")
        } else {
            Some(self.d_ui_currency.as_str())
        };

        // Start new transactions as income.
        let selected_type = if self.d_ui_type.is_empty() {
            Some("INCOME")
        } else {
            Some(self.d_ui_type.as_str())
        };

        // Use USD until a balance currency has been selected.
        let selected_balance_currency = if self.balance_currency.is_empty() {
            Some("USD")
        } else {
            Some(self.balance_currency.as_str())
        };

        // Pick the currency used by the balance totals.
        let balance_mode_picker = row![
            text("Mode:").size(12).color(colors.muted),
            pick_list(CURRENCIES, selected_balance_currency, |currency| {
                Message::BalanceCurrencyChanged(currency.to_string())
            })
            .width(Length::Fixed(110.0))
            .style(
                move |_theme: &Theme, _status| iced::widget::pick_list::Style {
                    text_color: colors.text,
                    placeholder_color: colors.muted,
                    handle_color: colors.text,
                    background: colors.panel.into(),
                    border: Border {
                        color: colors.divider,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                }
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // Keep the app title, balance mode, and Settings button together.
        let header = row![
            text("RScashm").size(30).color(colors.text),
            Space::new().width(Length::Fill),
            balance_mode_picker,
            button(
                text(if self.show_settings_panel {
                    "Close Settings"
                } else {
                    "Settings"
                })
                .color(colors.text)
            )
            .on_press(Message::ToggleSettingsPanel)
            .padding([6, 12])
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(colors.panel.into()),
                text_color: colors.text,
                border: Border {
                    color: colors.divider,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                shadow: Shadow::default(),
                ..button::Style::default()
            }),
        ]
        .align_y(Alignment::Center);

        // Parse the selected balance currency for transaction matching.
        let selected_currency_enum: Currency =
            self.balance_currency.parse().unwrap_or(Currency::USD);

        // Track income and expenses for the selected currency only.
        let mut total_income: i128 = 0;
        let mut total_expense: i128 = 0;
        // Ignore transactions saved in other currencies.
        for d in &self.data.v_data {
            if d.d_currency != selected_currency_enum {
                continue;
            }
            match normalize_transaction_type(&d.d_type) {
                Some("INCOME") => total_income = total_income.saturating_add(d.d_amount),
                Some("EXPENSE") => total_expense = total_expense.saturating_add(d.d_amount),
                _ => {}
            }
        }
        // Subtract expenses without overflowing the amount type.
        let net = total_income.saturating_sub(total_expense);
        // Use green for zero or positive totals and red for negative totals.
        let net_color = if net >= 0 {
            Color::from_rgb(0.2, 0.8, 0.3)
        } else {
            Color::from_rgb(0.9, 0.2, 0.2)
        };

        // Center the balance label above its formatted amount.
        let net_balance = column![
            text("Net Balance").size(14).color(colors.heading),
            text(to_currency(net, selected_currency_enum))
                .size(28)
                .color(net_color),
        ]
        .spacing(4)
        .align_x(Alignment::Center)
        .width(Length::Fill);

        // Show the income and expense totals in matching columns.
        let income_expense = row![
            // This column pairs the income label with its total.
            column![
                text("Total Income").size(14).color(colors.heading),
                text(int_to_text(total_income)).color(Color::from_rgb(0.2, 0.8, 0.3)),
            ]
            .width(Length::Fill)
            .align_x(Alignment::Center),
            // This column pairs the expense label with its total.
            column![
                text("Total Expense").size(14).color(colors.heading),
                text(int_to_text(total_expense)).color(Color::from_rgb(0.9, 0.2, 0.2)),
            ]
            .width(Length::Fill)
            .align_x(Alignment::Center),
        ]
        .width(Length::Fill);

        // The form contains a name, amount and currency, type and date, then Submit.
        let form = column![
            text("Add transaction").size(18).color(colors.text),
            text_input("Transaction name", &self.d_ui_name)
                .on_input(Message::NameChanged)
                .style(input_style),
            row![
                text_input("Amount (whole number)", &self.d_ui_amount)
                    .on_input(Message::AmountChanged)
                    .style(input_style),
                text("Currency").size(16).color(colors.text),
                pick_list(CURRENCIES, selected_currency, |currency| {
                    Message::CurrencyChanged(currency.to_string())
                }),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                pick_list(TRANSACTION_TYPES, selected_type, |ty| {
                    Message::TypeChanged(ty.to_string())
                }),
                text_input("Date (optional)", &self.d_ui_date)
                    .on_input(Message::DateChanged)
                    .style(input_style),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            button(text("Submit").color(Color::WHITE))
                .on_press(Message::TransmitData)
                .padding([10, 16])
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(Color::from_rgb(0.20, 0.38, 0.30).into()),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::from_rgb(0.31, 0.58, 0.45),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    shadow: Shadow::default(),
                    ..button::Style::default()
                }),
        ]
        .spacing(16)
        .width(Length::Fill);

        // Build the transaction rows before placing them in the scroll area.
        let mut list = column![].spacing(6).width(Length::Fill);
        for (i, tx) in self.data.v_data.iter().enumerate() {
            // Accept both current and legacy transaction type names.
            let transaction_type = normalize_transaction_type(&tx.d_type);
            // Mark income and expense amounts with the matching sign.
            let sign = match transaction_type {
                Some("INCOME") => "+",
                Some("EXPENSE") => "-",
                _ => "",
            };
            // Give income and expense amounts distinct colors.
            let amount_color = match transaction_type {
                Some("INCOME") => Color::from_rgb(0.2, 0.8, 0.3),
                Some("EXPENSE") => Color::from_rgb(0.9, 0.2, 0.2),
                _ => Color::from_rgb(0.7, 0.7, 0.7),
            };

            // Each history row shows the name, date, amount, and delete action.
            let row_item = container(
                row![
                    column![
                        text(&tx.d_name).color(colors.text),
                        text(&tx.d_date)
                            .size(12)
                            .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    ]
                    .width(Length::Fill),
                    text(format!(
                        "{sign} {}",
                        to_currency(tx.d_amount, tx.d_currency)
                    ))
                    .color(amount_color),
                    button(text("Delete").color(Color::WHITE))
                        .on_press(Message::RemoveData(i))
                        .style(|_theme: &Theme, _status| button::Style {
                            background: Some(Color::from_rgb(0.45, 0.15, 0.15).into()),
                            text_color: Color::WHITE,
                            border: Border {
                                color: Color::from_rgb(0.6, 0.25, 0.25),
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            shadow: Shadow::default(),
                            ..button::Style::default()
                        }),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding(9)
            .style(move |_theme: &Theme| container::Style {
                background: Some(colors.row.into()),
                border: Border {
                    color: colors.row_border,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..container::Style::default()
            });

            list = list.push(row_item);
        }

        let encryption_status = match (self.is_encrypted, self.encryption_unlocked) {
            (true, true) => "Encrypted / unlocked",
            (true, false) => "Encrypted / locked",
            (false, _) => "Plaintext",
        };
        let encryption_action = if self.is_encrypted && !self.encryption_unlocked {
            "Unlock"
        } else if self.is_encrypted {
            "Re-encrypt"
        } else {
            "Encrypt"
        };
        let lock_button: Element<'_, Message> = if self.is_encrypted && self.encryption_unlocked {
            button(text("Lock"))
                .on_press(Message::LockEncryption)
                .into()
        } else {
            Space::new().into()
        };
        let mut history_header = column![
            row![text("Transaction History").size(20).color(colors.text)].width(Length::Fill),
            row![
                text(encryption_status).size(12).color(colors.muted),
                text_input("Password", &self.encryption_password)
                    .secure(true)
                    .on_input(|password| {
                        Message::EncryptionPasswordChanged(Zeroizing::new(password))
                    })
                    .width(Length::Fill),
                button(text("Check").color(colors.text)).on_press(Message::CheckEncryption),
                button(text(encryption_action).color(colors.text))
                    .on_press(Message::ApplyEncryption),
                lock_button,
                button(text("Remove All").color(colors.text)).on_press(Message::RemoveAllData),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8);
        if self.data.v_data.is_empty() {
            history_header =
                history_header.push(text("No transactions yet").size(13).color(colors.muted));
        }

        // Keep the summary fixed while the transaction list scrolls.
        let history = scrollable(list).width(Length::Fill).height(Length::Fill);

        // Show errors in red and routine updates in a muted color.
        let status_color =
            if self.status.starts_with("Failed") || self.status.starts_with("Couldn't") {
                Color::from_rgb(0.9, 0.2, 0.2)
            } else {
                colors.muted
            };
        // Stack the fixed header and summary above the form and history.
        let mut page = column![header]
            .spacing(16)
            .align_x(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill);
        if !self.status.is_empty() {
            page = page.push(text(&self.status).size(12).color(status_color));
        }

        page.push(net_balance)
            .push(income_expense)
            .push(form)
            .push(history_header)
            .push(history)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::{CURRENCIES, UiColors, readable_text_color};
    use crate::data::{AppTheme, Currency, RgbColor};
    use iced::Color;

    // Check that every picker currency can be parsed.
    #[test]
    fn every_currency_option_can_be_used_as_balance_mode() {
        for currency_code in CURRENCIES {
            assert!(
                currency_code.parse::<Currency>().is_ok(),
                "currency code {currency_code} is missing from Currency"
            );
        }
    }

    // Dark backgrounds should not hide black custom text.
    #[test]
    fn black_custom_text_stays_readable_on_dark_background() {
        assert_eq!(
            readable_text_color(Color::BLACK, Color::BLACK),
            Color::WHITE
        );
    }

    // Light backgrounds can keep dark custom text as-is.
    #[test]
    fn dark_custom_text_stays_readable_on_light_background() {
        assert_eq!(
            readable_text_color(Color::BLACK, Color::WHITE),
            Color::BLACK
        );
    }

    // Keep a custom text color when its contrast is already strong enough.
    #[test]
    fn custom_text_color_is_kept_when_it_has_enough_contrast() {
        let colors = UiColors::for_theme(
            AppTheme::Dark,
            None,
            Some(RgbColor {
                red: 255,
                green: 0,
                blue: 0,
            }),
        );

        assert_eq!(colors.text, Color::from_rgb8(255, 0, 0));
    }

    // A black custom background needs light text in the White theme.
    #[test]
    fn white_theme_keeps_text_readable_on_custom_black_background() {
        let colors = UiColors::for_theme(
            AppTheme::Light,
            Some(RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            }),
            None,
        );

        assert_eq!(colors.text, Color::WHITE);
    }
}
