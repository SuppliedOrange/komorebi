use crate::config::LabelPrefix;
use crate::render::RenderConfig;
use crate::widget::BarWidget;
use eframe::egui::{Context, Label, TextFormat, Ui, Align, text::LayoutJob};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use windows::Win32::Foundation::LPARAM;
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, GetWindowTextLengthW};
use std::sync::Mutex;

static FOUND_TITLE: Mutex<Option<String>> = Mutex::new(None);

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DiscordConfig {
    pub enable: bool,
    pub data_refresh_interval: Option<u64>,
    pub label_prefix: Option<LabelPrefix>,
    pub window_filter_keyword: Option<String>,
}

impl From<DiscordConfig> for Discord {
    fn from(value: DiscordConfig) -> Self {
        Self {
            enable: value.enable,
            data_refresh_interval: value.data_refresh_interval.unwrap_or(2),
            label_prefix: value.label_prefix.unwrap_or(LabelPrefix::IconAndText),
            window_filter_keyword: value.window_filter_keyword.unwrap_or("Discord".to_string()),
            last_updated: Instant::now(),
            notification_count: 0,
        }
    }
}

pub struct Discord {
    pub enable: bool,
    data_refresh_interval: u64,
    label_prefix: LabelPrefix,
    window_filter_keyword: String,
    last_updated: Instant,
    notification_count: u32,
}

impl Discord {

    /*
    Iterate over all windows, find the one with the title containing "Discord"
    or the specified keyword.
    */
    fn get_discord_window_title(&self) -> Option<String> {
        unsafe {

            *FOUND_TITLE.lock().unwrap() = None;

            unsafe extern "system" fn enum_callback(
                window: windows::Win32::Foundation::HWND,
                lparam: LPARAM,
            ) -> windows::Win32::Foundation::BOOL {
                let length = GetWindowTextLengthW(window);
                if length == 0 {
                    return true.into();
                }

                let mut buffer = vec![0u16; length as usize + 1];
                let chars_copied = GetWindowTextW(window, &mut buffer);
                if chars_copied == 0 {
                    return true.into();
                }

                let title = String::from_utf16_lossy(&buffer[..chars_copied as usize]);

                let filter_keyword = unsafe { &*(lparam.0 as *const String) };
                if title.contains(filter_keyword) {
                    *FOUND_TITLE.lock().unwrap() = Some(title);
                }

                true.into()
            }

            EnumWindows(Some(enum_callback), LPARAM(&self.window_filter_keyword as *const _ as isize)).ok()?;

            let result = FOUND_TITLE.lock().unwrap().clone();
            result
        }
    }
    
    /*
    Parse the window title. It's usually in the format of "(count) Discord | activity",
    or "Discord | activity" if there are no notifications.
    */
    fn parse_notification_count(&self, title: &str) -> u32 {
        if title.starts_with('(') {
            if let Some(end_bracket) = title.find(')') {
                let count_str = &title[1..end_bracket];
                if let Ok(count) = count_str.parse::<u32>() {
                    return count;
                }
            }
        }
        0
    }

    fn output(&mut self) -> String {
        let now = Instant::now();
        if now.duration_since(self.last_updated) > Duration::from_secs(self.data_refresh_interval) {
            if let Some(title) = self.get_discord_window_title() {
                self.notification_count = self.parse_notification_count(&title);
            } else {
                self.notification_count = 0;
            }
            self.last_updated = now;
        }

        match self.label_prefix {
            LabelPrefix::Text | LabelPrefix::IconAndText => {
                if self.notification_count > 0 {
                    format!("DISC {}", self.notification_count)
                } else {
                    "DISC".to_string()
                }
            }
            LabelPrefix::None | LabelPrefix::Icon => {
                if self.notification_count > 0 {
                    self.notification_count.to_string()
                } else {
                    String::new()
                }
            }
        }
    }
}

impl BarWidget for Discord {
    fn render(&mut self, ctx: &Context, ui: &mut Ui, config: &mut RenderConfig) {
        if self.enable {
            let output = self.output();
            let mut layout_job = LayoutJob::simple(
                match self.label_prefix {
                    LabelPrefix::Icon | LabelPrefix::IconAndText => {
                        egui_phosphor::regular::DISCORD_LOGO.to_string()
                    }
                    LabelPrefix::None | LabelPrefix::Text => String::new(),
                },
                config.icon_font_id.clone(),
                ctx.style().visuals.selection.stroke.color,
                100.0,
            );

            layout_job.append(
                &output,
                10.0,
                TextFormat {
                    font_id: config.text_font_id.clone(),
                    color: ctx.style().visuals.text_color(),
                    valign: Align::Center,
                    ..Default::default()
                },
            );

            config.apply_on_widget(false, ui, |ui| {
                ui.add(Label::new(layout_job).selectable(false));
            });
        }
    }
}